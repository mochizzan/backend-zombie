use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct UserInventory {
    pub id: i64,
    pub user_id: i64,
    pub shop_item_id: i64,
    pub equipped: bool,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct PurchaseItem {
    pub shop_item_id: i64,
}
