use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct LeaderboardRow {
    pub id: i64,
    pub user_id: i64,
    pub category: String,
    pub score: i64,
    pub rank: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub rank: i32,
    pub user_id: i64,
    pub player_id: String,
    pub username: String,
    pub nickname: String,
    pub score: i64,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub category: String,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[derive(Debug, Serialize)]
pub struct PlayerRank {
    pub rank: i32,
    pub score: i64,
    pub total_players: i64,
}
