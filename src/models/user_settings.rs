use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, Serialize, FromRow)]
pub struct UserSettings {
    pub id: i64,
    pub user_id: i64,
    pub setting_key: String,
    pub setting_value: u8,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettings {
    pub settings: HashMap<String, u8>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSingleSetting {
    pub value: u8,
}
