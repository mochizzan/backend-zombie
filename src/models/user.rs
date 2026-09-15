use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub player_id: String,
    pub username: String,
    pub nickname: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 1, max = 64))]
    pub player_id: String,
    #[validate(length(min = 1, max = 64))]
    pub username: String,
    #[validate(length(min = 1, max = 64))]
    pub nickname: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub role: Option<String>,
}
