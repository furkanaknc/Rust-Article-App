use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};
use chrono::NaiveDateTime;



#[derive(Deserialize)]
pub struct CreateArticleBody {
    pub title: String,
    pub content: String,
}

#[derive(Serialize, FromRow)]
pub struct Article {
   pub id: i32,
   pub title: String,
   pub content: String,
   pub published_by: i32,
   pub published_on: Option<NaiveDateTime>,
}


#[derive(Deserialize)]
pub struct UpdateArticleBody {
    pub title: Option<String>,
    pub content: Option<String>,
}