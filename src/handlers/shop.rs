use actix_web::{web, HttpResponse};
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::shop_item::{CreateShopItem, UpdateShopItem};
use crate::models::{success_response, created_response, no_content_response, list_response};
use crate::services;

#[derive(serde::Deserialize)]
pub struct ShopQuery {
    pub role: Option<String>,
    pub category: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_items(
    pool: web::Data<DbPool>,
    query: web::Query<ShopQuery>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(20).min(100);
    let (items, cursor, has_more) = services::shop::list_items(
        pool.get_ref(),
        query.role.as_deref(),
        query.category.as_deref(),
        query.cursor.as_deref(),
        limit,
    )
    .await?;
    Ok(list_response(items, cursor, has_more))
}

pub async fn get_item(
    pool: web::Data<DbPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let item = services::shop::get_item(pool.get_ref(), path.into_inner()).await?;
    Ok(success_response(item))
}

pub async fn create_item(
    pool: web::Data<DbPool>,
    body: web::Json<CreateShopItem>,
) -> Result<HttpResponse, AppError> {
    let item = services::shop::create_item(pool.get_ref(), body.into_inner()).await?;
    Ok(created_response(item))
}

pub async fn update_item(
    pool: web::Data<DbPool>,
    path: web::Path<i64>,
    body: web::Json<UpdateShopItem>,
) -> Result<HttpResponse, AppError> {
    let item =
        services::shop::update_item(pool.get_ref(), path.into_inner(), body.into_inner()).await?;
    Ok(success_response(item))
}

pub async fn delete_item(
    pool: web::Data<DbPool>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    services::shop::delete_item(pool.get_ref(), path.into_inner()).await?;
    Ok(no_content_response())
}
