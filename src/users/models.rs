use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};

#[derive(Deserialize)]
pub struct CreateUserBody {
    pub username: String,
    pub password: String,
    pub email:String,
}

#[derive(Serialize, FromRow)]
pub struct UserNoPassword {
    pub id: i32,
    pub username: String,
    pub email:String,
    pub role:String,
}


#[derive(Serialize, FromRow)]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role:String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUserBody {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}



