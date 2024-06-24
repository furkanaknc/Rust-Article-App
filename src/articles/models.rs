use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
use chrono::NaiveDateTime;
use utoipa::ToSchema;



#[derive(Deserialize, ToSchema)]
pub struct CreateArticleBody {
    #[schema(example="title", required=true)]
    pub title: String,
    #[schema(example="content", required=true)]
    pub content: String,
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct Article {
   pub id: i32,
   pub title: String,
   pub content: String,
   pub published_by: i32,
   pub published_on: Option<NaiveDateTime>,
}


#[derive(Deserialize, ToSchema)]
pub struct UpdateArticleBody {
    #[schema(example="title", required=true)]
    pub title: Option<String>,
    #[schema(example="content", required=true)]
    pub content: Option<String>,
}