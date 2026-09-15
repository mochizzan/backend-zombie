use actix_web::{web, HttpResponse};
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::user_inventory::PurchaseItem;
use crate::models::{success_response, no_content_response, list_response};
use crate::services;

#[derive(serde::Deserialize)]
pub struct InventoryQuery {
    pub category: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

/// GET /users/:player_id/inventory
/// List inventory items with optional category filter and cursor pagination.
pub async fn list_inventory(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    query: web::Query<InventoryQuery>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let limit = query.limit.unwrap_or(20).min(100);
    let (items, cursor, has_more) = services::inventory::list_inventory(
        pool.get_ref(),
        &player_id,
        query.category.as_deref(),
        query.cursor.as_deref(),
        limit,
    )
    .await?;
    Ok(list_response(items, cursor, has_more))
}

/// POST /users/:player_id/inventory
/// Purchase a shop item for the given player.
pub async fn purchase_item(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    body: web::Json<PurchaseItem>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let inv = services::inventory::purchase_item(pool.get_ref(), &player_id, body.into_inner())
        .await?;
    Ok(success_response(inv))
}

/// DELETE /users/:player_id/inventory/:item_id
/// Remove an inventory item (soft delete).
pub async fn remove_item(
    pool: web::Data<DbPool>,
    path: web::Path<(String, i64)>,
) -> Result<HttpResponse, AppError> {
    let (player_id, inventory_id) = path.into_inner();
    services::inventory::remove_item(pool.get_ref(), &player_id, inventory_id).await?;
    Ok(no_content_response())
}

/// GET /users/:player_id/inventory/has/:shop_item_id
/// Check if a player owns a specific shop item.
pub async fn has_item(
    pool: web::Data<DbPool>,
    path: web::Path<(String, i64)>,
) -> Result<HttpResponse, AppError> {
    let (player_id, shop_item_id) = path.into_inner();
    let owned =
        services::inventory::has_item(pool.get_ref(), &player_id, shop_item_id).await?;
    Ok(success_response(serde_json::json!({
        "player_id": player_id,
        "shop_item_id": shop_item_id,
        "owned": owned,
    })))
}
