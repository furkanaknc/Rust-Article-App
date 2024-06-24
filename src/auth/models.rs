use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::fmt;
use utoipa::ToSchema;

pub struct AppState {
    pub db: Pool<Postgres>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct TokenClaims {
    pub id: i32,
    pub role: String,
}

impl fmt::Debug for TokenClaims {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenClaims")
            .field("id", &self.id)
            .field("role", &self.role)
            .finish()
    }
}