use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
use utoipa::ToSchema;
#[derive(Deserialize,ToSchema)]
pub struct CreateUserBody {
    #[schema(example="username", required=true)]
    pub username: String,
    #[schema(example="email@email.com", required=true)]
    pub password: String,
    #[schema(example="password", required=true)]
    pub email:String,
}

#[derive(Serialize, FromRow,ToSchema)]
pub struct UserNoPassword {
    pub id: i32,
    #[schema(example="username", required=true)]
    pub username: String,
    #[schema(example="email@email.com", required=true)]
    pub email:String,
    #[schema(example="password", required=true)]
    pub role:String,
}


#[derive(Serialize, FromRow,ToSchema)]
pub struct AuthUser {
    #[schema(example="id", required=false)]
    pub id: i32,
    #[schema(example="username", required=true)]
    pub username: String,
    #[schema(example="password", required=true)]
    pub password: String,
    #[schema(example="role", required=false)]
    pub role:String,
}

#[derive(Serialize, Deserialize,ToSchema)]
pub struct UpdateUserBody {
    #[schema(example="username", required=true)]
    pub username: Option<String>,
    #[schema(example="email@email.com", required=true)]
    pub email: Option<String>,
    #[schema(example="password", required=true)]
    pub password: Option<String>,
}



