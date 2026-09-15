use actix_web::{web, HttpResponse};

use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::user_stats::UpdateStats;
use crate::models::success_response;
use crate::services;

pub async fn get_stats(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let stats = services::user_stats::get_stats(pool.get_ref(), &player_id).await?;
    Ok(success_response(stats))
}

pub async fn update_stats(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    body: web::Json<UpdateStats>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let stats =
        services::user_stats::update_stats(pool.get_ref(), &player_id, body.into_inner()).await?;
    Ok(success_response(stats))
}

pub async fn update_online(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    services::user_stats::update_online(pool.get_ref(), &player_id).await?;
    Ok(success_response("ok"))
}

pub async fn update_join(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    services::user_stats::update_join(pool.get_ref(), &player_id).await?;
    Ok(success_response("ok"))
}
