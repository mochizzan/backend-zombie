pub mod user;
pub mod user_stats;
pub mod shop_item;
pub mod user_inventory;
pub mod user_settings;
pub mod leaderboard;

use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiMeta {
    pub cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: Option<ApiMeta>,
}

pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": data,
    }))
}

pub fn created_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Created().json(serde_json::json!({
        "success": true,
        "data": data,
    }))
}

pub fn no_content_response() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

pub fn list_response<T: Serialize>(data: Vec<T>, cursor: Option<String>, has_more: bool) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": data,
        "meta": {
            "cursor": cursor,
            "has_more": has_more,
        },
    }))
}
