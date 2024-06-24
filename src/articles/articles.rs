use crate::{AppState, TokenClaims};
use actix_web::{
    get, post,delete,put,
    web::{Data, Json, ReqData, Path},
    HttpResponse, Responder,
};
use sqlx::{self};
use crate::articles::models::{CreateArticleBody, Article, UpdateArticleBody};

#[utoipa::path(
    request_body = CreateArticleBody,
    responses(
        (status = 200, description = "Create an article", body = CreateArticleBody),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/article")]
async fn create_article(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    body: Json<CreateArticleBody>,
) -> impl Responder {
    match req_user {
        Some(user) => {
            let article: CreateArticleBody = body.into_inner();

            match sqlx::query_as::<_, Article>(
                "INSERT INTO articles (title, content, published_by)
                VALUES ($1, $2, $3)
                RETURNING id, title, content, published_by, published_on",
            )
            .bind(article.title)
            .bind(article.content)
            .bind(user.id)
            .fetch_one(&state.db)
            .await
            {
                Ok(articles) => HttpResponse::Ok().json(articles),
                Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
            }
        }
        _ => HttpResponse::Unauthorized().json("Unable to verify identity"),
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "get all articles", body = Article),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/article/{id}")]
async fn get_article(
    state: Data<AppState>,
    article_id: Path<i32>,
) -> impl Responder {
    let article_id = article_id.into_inner();

    match sqlx::query_as::<_, Article>(
        "SELECT id, title, content, published_by, published_on FROM articles WHERE id = $1"
    )
    .bind(article_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(article) => HttpResponse::Ok().json(article),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json("Article not found"),
        Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "get an articles", body = Article),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/articles")]
async fn get_all_articles(
    state: Data<AppState>,
) -> impl Responder {
    match sqlx::query_as::<_, Article>(
        "SELECT id, title, content, published_by, published_on FROM articles"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(articles) => HttpResponse::Ok().json(articles),
        Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
    }
}

#[utoipa::path(
    request_body = Article,
    responses(
        (status = 200, description = "Delete an article", body = Article),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[delete("/article/{id}")]
async fn delete_article(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    article_id: Path<i32>,
) -> impl Responder {
    if let Some(user) = req_user {
        let article_id = article_id.into_inner();

        match sqlx::query_as::<_, (i32,)>(
            "SELECT published_by FROM articles WHERE id = $1"
        )
        .bind(article_id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some((published_by,))) => {
                if published_by == user.id || user.role == "admin" {
                    match sqlx::query(
                        "DELETE FROM articles WHERE id = $1"
                    )
                    .bind(article_id)
                    .execute(&state.db)
                    .await
                    {
                        Ok(_) => HttpResponse::Ok().json("Article deleted successfully"),
                        Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
                    }
                } else {
                    HttpResponse::Forbidden().json("You can only delete your own articles")
                }
            }
            Ok(None) => HttpResponse::NotFound().json("Article not found"),
            Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
        }
    } else {
        HttpResponse::Unauthorized().json("Unable to verify identity")
    }
}

#[utoipa::path(
    request_body = UpdateArticleBody,
    responses(
        (status = 200, description = "Update an article's title", body = UpdateArticleBody),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[put("/article/{id}/title")]
async fn update_article_title(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    article_id: Path<i32>,
    body: Json<UpdateArticleBody>,
) -> impl Responder {
    if let Some(user) = req_user {
        let article_id = article_id.into_inner();
        let updated_article = body.into_inner();

        match sqlx::query_as::<_, (i32,)>("SELECT published_by FROM articles WHERE id = $1")
            .bind(&article_id)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some((published_by,))) => {
                if published_by == user.id || user.role == "admin" {
                    match sqlx::query(
                        "UPDATE articles SET title = $1 WHERE id = $2"
                    )
                    .bind(&updated_article.title)
                    .bind(&article_id)
                    .execute(&state.db)
                    .await
                    {
                        Ok(_) => {
                            match sqlx::query_as::<_, Article>(
                                "SELECT id, title, content, published_by, published_on FROM articles WHERE id = $1"
                            )
                            .bind(&article_id)
                            .fetch_one(&state.db)
                            .await
                            {
                                Ok(updated_article) => HttpResponse::Ok().json(updated_article),
                                Err(error) => HttpResponse::InternalServerError().json(format!("Failed to fetch updated article: {:?}", error)),
                            }
                        },
                        Err(error) => HttpResponse::InternalServerError().json(format!("Failed to update title: {:?}", error)),
                    }
                } else {
                    HttpResponse::Forbidden().json("You can only update your own articles")
                }
            },
            Ok(None) => HttpResponse::NotFound().json("Article not found"),
            Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
        }
    } else {
        HttpResponse::Unauthorized().json("Unable to verify identity")
    }
}

#[utoipa::path(
    request_body = UpdateArticleBody,
    responses(
        (status = 200, description = "Update an article's content", body = UpdateArticleBody),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error.")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[put("/article/{id}/content")]
async fn update_article_content(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    article_id: Path<i32>,
    body: Json<UpdateArticleBody>,
) -> impl Responder {
    if let Some(user) = req_user {
        let article_id = article_id.into_inner();
        let updated_article = body.into_inner();

        match sqlx::query_as::<_, (i32,)>("SELECT published_by FROM articles WHERE id = $1")
            .bind(&article_id)
            .fetch_optional(&state.db)
            .await
        {
            Ok(Some((published_by,))) => {
                if published_by == user.id || user.role == "admin" {
                    match sqlx::query(
                        "UPDATE articles SET content = $1 WHERE id = $2"
                    )
                    .bind(&updated_article.content)
                    .bind(&article_id)
                    .execute(&state.db)
                    .await
                    {
                        Ok(_) => {
                            match sqlx::query_as::<_, Article>(
                                "SELECT id, title, content, published_by, published_on FROM articles WHERE id = $1"
                            )
                            .bind(&article_id)
                            .fetch_one(&state.db)
                            .await
                            {
                                Ok(updated_article) => HttpResponse::Ok().json(updated_article),
                                Err(error) => HttpResponse::InternalServerError().json(format!("Failed to fetch updated article: {:?}", error)),
                            }
                        },
                        Err(error) => HttpResponse::InternalServerError().json(format!("Failed to update content: {:?}", error)),
                    }
                } else {
                    HttpResponse::Forbidden().json("You can only update your own articles")
                }
            },
            Ok(None) => HttpResponse::NotFound().json("Article not found"),
            Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
        }
    } else {
        HttpResponse::Unauthorized().json("Unable to verify identity")
    }
}