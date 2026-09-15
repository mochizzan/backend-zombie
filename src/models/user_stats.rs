use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct UserStats {
    pub id: i64,
    pub user_id: i64,
    pub exp: i64,
    pub fang: i64,
    pub bcoin: i64,
    pub level: i32,
    pub total_playtime: i64,
    pub matches_played: i32,
    pub total_spent: i64,
    pub version: i32,
    pub last_online_at: Option<NaiveDateTime>,
    pub last_join_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStats {
    pub exp: Option<i64>,
    pub fang: Option<i64>,
    pub bcoin: Option<i64>,
    pub level: Option<i32>,
    pub total_playtime: Option<i64>,
    pub matches_played: Option<i32>,
    pub version: Option<i32>,
}
