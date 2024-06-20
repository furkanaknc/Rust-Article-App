use bcrypt::hash;
use sqlx::PgPool;
use sqlx::postgres::PgQueryResult;
use std::env;

pub async fn seed_admin_user(pool: &PgPool) -> Result<PgQueryResult, sqlx::Error> {
    let admin_username = env::var("ADMIN_USERNAME").expect("ADMIN_USERNAME must be set");
    let admin_password = env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");
    let admin_email = env::var("ADMIN_EMAIL").expect("ADMIN_EMAIL must be set");

    let hashed_password = hash(admin_password, bcrypt::DEFAULT_COST).unwrap();

    sqlx::query(
        "INSERT INTO users (username, password, email, role)
         VALUES ($1, $2, $3, 'admin')
         ON CONFLICT (username) DO NOTHING"
    )
    .bind(admin_username)
    .bind(hashed_password)
    .bind(admin_email)
    .execute(pool)
    .await
}