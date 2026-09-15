use actix_web::{web, HttpResponse};

use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::user_settings::{UpdateSettings, UpdateSingleSetting};
use crate::models::{no_content_response, success_response};
use crate::services;

pub async fn get_settings(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let settings = services::settings::get_settings(pool.get_ref(), &player_id).await?;
    Ok(success_response(settings))
}

pub async fn update_settings(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    body: web::Json<UpdateSettings>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let settings = services::settings::update_settings(pool.get_ref(), &player_id, body.settings.clone())
        .await?;
    Ok(success_response(settings))
}

pub async fn update_single(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
    body: web::Json<UpdateSingleSetting>,
) -> Result<HttpResponse, AppError> {
    let (player_id, key) = path.into_inner();
    services::settings::update_single(pool.get_ref(), &player_id, &key, body.value).await?;
    Ok(success_response("ok"))
}

pub async fn delete_setting(
    pool: web::Data<DbPool>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let (player_id, key) = path.into_inner();
    services::settings::delete_setting(pool.get_ref(), &player_id, &key).await?;
    Ok(no_content_response())
}
