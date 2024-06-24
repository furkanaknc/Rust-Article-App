use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use utoipa::OpenApi;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify,
};
use utoipa_swagger_ui::SwaggerUi;

mod users;
use users::{login, register, update_email, update_password, update_username};

mod articles;
use articles::{
    create_article, delete_article, get_all_articles, get_article, update_article_content,
    update_article_title,
};

mod auth;
use auth::{validator, AppState, TokenClaims};

mod seed;
use seed::seed_admin_user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");

    seed_admin_user(&pool)
        .await
        .expect("Failed to seed admin user");

    #[derive(OpenApi)]
    #[openapi(
        paths(
           articles::articles::create_article,
           articles::articles::get_article,
           articles::articles::get_all_articles,
           articles::articles::delete_article,
           articles::articles::update_article_content,
           articles::articles::update_article_title,
           users::users::register,
           users::users::login,
           users::users::update_email,
           users::users::update_username,
           users::users::update_password,
        ),
        components(
            schemas(
                TokenClaims,
                articles::models::CreateArticleBody,
                articles::models::Article,
                articles::models::UpdateArticleBody,
                users::models::CreateUserBody,
                users::models::UserNoPassword,
                users::models::AuthUser,
                users::models::UpdateUserBody,
            )
        ),
        tags(
            (name = "APIs", description = "API management endpoints")
        ),
        modifiers(&SecurityModifier)
    )]

    struct ApiDoc;
    struct SecurityModifier;
    impl Modify for SecurityModifier {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap();
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
            components.add_security_scheme(
                "login",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Basic).build()),
            );
        }
    }

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .app_data(Data::new(AppState { db: pool.clone() }))
            .service(login)
            .service(register)
            .service(
                web::scope("")
                    .wrap(bearer_middleware)
                    .service(create_article)
                    .service(get_all_articles)
                    .service(get_article)
                    .service(delete_article)
                    .service(update_article_content)
                    .service(update_article_title)
                    .service(update_email)
                    .service(update_password)
                    .service(update_username),
            )
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
