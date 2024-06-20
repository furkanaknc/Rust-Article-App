use crate::users::models::{AuthUser, CreateUserBody, UpdateUserBody, UserNoPassword};
use crate::{AppState, TokenClaims};
use actix_web::{
    get, post, put, web,
    web::{Data, Json, Path, ReqData},
    HttpResponse, Responder,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use bcrypt::{hash, verify, DEFAULT_COST};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use regex::Regex;
use sha2::Sha256;
use sqlx::{self, Error as SqlxError};

#[post("/register")]
async fn register(state: Data<AppState>, body: Json<CreateUserBody>) -> impl Responder {
    let user = body.into_inner();

    let email_regex = Regex::new(r"^[\w\.-]+@[\w\.-]+\.[a-zA-Z]{2,4}$").unwrap();
    if !email_regex.is_match(&user.email) {
        return HttpResponse::BadRequest().json("Invalid email format");
    }

    let hashed_password = hash(&user.password, DEFAULT_COST).unwrap();

    let username_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE username = $1)")
            .bind(&user.username)
            .fetch_one(&state.db)
            .await
            .unwrap_or(false);

    if username_exists {
        return HttpResponse::Conflict().json("Username already exists");
    }

    let email_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
            .bind(&user.email)
            .fetch_one(&state.db)
            .await
            .unwrap_or(false);

    if email_exists {
        return HttpResponse::Conflict().json("Email already exists");
    }

    match sqlx::query_as::<_, UserNoPassword>(
        "INSERT INTO users (username, password, email, role)
        VALUES ($1, $2, $3, 'user')
        RETURNING id, username, email, role",
    )
    .bind(user.username)
    .bind(hashed_password)
    .bind(user.email)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

#[get("/login")]
async fn login(state: Data<AppState>, credentials: BasicAuth) -> impl Responder {
    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set!")
            .as_bytes(),
    )
    .unwrap();

    let username = credentials.user_id();
    let password = credentials.password();

    match password {
        None => HttpResponse::Unauthorized().json("Must provide a valid password"),
        Some(pass) => {
            match sqlx::query_as::<_, AuthUser>(
                "SELECT id, username, password, role FROM users WHERE username = $1",
            )
            .bind(username.to_string())
            .fetch_one(&state.db)
            .await
            {
                Ok(user) => {
                    let is_valid = verify(pass, &user.password).unwrap();
                    if is_valid {
                        let claims = TokenClaims {
                            id: user.id,
                            role: user.role,
                        };

                        let token = claims.sign_with_key(&jwt_secret).unwrap();
                        HttpResponse::Ok().json(token)
                    } else {
                        HttpResponse::Unauthorized().json("Invalid credentials")
                    }
                }
                Err(_) => HttpResponse::InternalServerError().json("Error fetching user"),
            }
        }
    }
}

#[put("/user/{id}/username")]
async fn update_username(
    state: web::Data<AppState>,
    req_user: Option<web::ReqData<TokenClaims>>,
    user_id: web::Path<i32>,
    body: web::Json<UpdateUserBody>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let update_info = body.into_inner();

    if let Some(user) = req_user.as_ref().map(|data| data.clone().into_inner())  {
        if user.id != user_id && user.role != "admin" {
            return HttpResponse::Forbidden()
                .json("You can only update your own information or be an admin");
        }

        if let Some(username) = &update_info.username {
            if let Ok(_) = sqlx::query("SELECT id FROM users WHERE username = $1")
                .bind(username)
                .fetch_one(&state.db)
                .await
            {
                return HttpResponse::BadRequest().json("Username already exists");
            }

            let result = sqlx::query_as::<_, UserNoPassword>(
                "UPDATE users SET username = $1 WHERE id = $2 RETURNING id, username, email, role",
            )
            .bind(username)
            .bind(user_id)
            .fetch_one(&state.db)
            .await;

            match result {
                Ok(updated_user) => HttpResponse::Ok().json(updated_user),
                Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json("User not found"),
                Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
            }
        } else {
            HttpResponse::BadRequest().json("Username not provided")
        }
    } else {
        HttpResponse::Unauthorized().json("Unable to verify identity")
    }
}

#[put("/user/{id}/email")]
async fn update_email(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    user_id: Path<i32>,
    body: Json<UpdateUserBody>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let update_info = body.into_inner();

    if let Some(user) = req_user.as_ref().map(|data| data.clone().into_inner()) {
        if user.id != user_id && user.role != "admin" {
            return HttpResponse::Forbidden()
                .json("You can only update your own information or be an admin");
        }

        if let Some(email) = &update_info.email {
            let email_regex = Regex::new(r"^[\w\.-]+@[\w\.-]+\.[a-zA-Z]{2,4}$").unwrap();
            if !email_regex.is_match(email) {
                return HttpResponse::BadRequest().json("Invalid email format");
            }

            if let Ok(_) = sqlx::query("SELECT id FROM users WHERE email = $1")
                .bind(email)
                .fetch_one(&state.db)
                .await
            {
                return HttpResponse::BadRequest().json("Email already exists");
            }

            let result = sqlx::query_as::<_, UserNoPassword>(
                "UPDATE users SET email = $1 WHERE id = $2 RETURNING id, username, email, role",
            )
            .bind(email)
            .bind(user_id)
            .fetch_one(&state.db)
            .await;

            match result {
                Ok(updated_user) => HttpResponse::Ok().json(updated_user),
                Err(SqlxError::RowNotFound) => HttpResponse::NotFound().json("User not found"),
                Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
            }
        } else {
            HttpResponse::BadRequest().json("Email not provided")
        }
    } else {
        HttpResponse::Unauthorized().json("Unable to verify identity")
    }
}

#[put("/user/{id}/password")]
async fn update_password(
    state: Data<AppState>,
    req_user: Option<ReqData<TokenClaims>>,
    user_id: Path<i32>,
    body: Json<UpdateUserBody>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    let update_info = body.into_inner();

    if let Some(user) = req_user.as_ref().map(|data| data.clone().into_inner()) {
        if user.id != user_id && user.role != "admin" {
            return HttpResponse::Forbidden()
                .json("You can only update your own information or be an admin");
        }

        if let Some(password) = update_info.password {
            let hashed_password = hash(&password, DEFAULT_COST).unwrap();

            let result = sqlx::query_as::<_, UserNoPassword>(
                "UPDATE users SET password = $1 WHERE id = $2 RETURNING id, username, email, role",
            )
            .bind(hashed_password)
            .bind(user_id)
            .fetch_one(&state.db)
            .await;

            match result {
                Ok(updated_user) => HttpResponse::Ok().json(updated_user),
                Err(SqlxError::RowNotFound) => HttpResponse::NotFound().json("User not found"),
                Err(error) => HttpResponse::InternalServerError().json(format!("Database error: {:?}", error)),
            }
        } else {
            HttpResponse::BadRequest().json("Password not provided")
        }
    } else {
        HttpResponse::Unauthorized().json("Unable to verify identity")
    }
}


