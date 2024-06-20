use actix_web::{
    web::{self, Data}, App, HttpServer
};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
mod users;
use users::{login, register, update_email,update_password,update_username};

mod articles;
use articles::{create_article, delete_article, get_all_articles, get_article, update_article_content,update_article_title};

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

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        App::new()
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
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
