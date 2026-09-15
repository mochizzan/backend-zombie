use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ShopItem {
    pub id: i64,
    pub role: String,
    pub category: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub discount_percent: Option<i32>,
    pub discounted_price: i64,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShopItem {
    pub role: String,
    pub category: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub discount_percent: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShopItem {
    pub role: Option<String>,
    pub category: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub discount_percent: Option<i32>,
    pub is_active: Option<bool>,
}
