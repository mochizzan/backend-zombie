use actix_web::{web, HttpResponse};
use validator::Validate;

use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::{created_response, no_content_response, success_response};
use crate::models::user::{CreateUser, UpdateUser};
use crate::services;

pub async fn register(
    pool: web::Data<DbPool>,
    body: web::Json<CreateUser>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let user = services::user::create_user(pool.get_ref(), body.into_inner()).await?;
    Ok(created_response(user))
}

pub async fn get_profile(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let user = services::user::get_user(pool.get_ref(), &player_id).await?;
    Ok(success_response(user))
}

pub async fn update_profile(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    body: web::Json<UpdateUser>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    let user =
        services::user::update_user(pool.get_ref(), &player_id, body.into_inner()).await?;
    Ok(success_response(user))
}

pub async fn delete_player(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let player_id = path.into_inner();
    services::user::delete_user(pool.get_ref(), &player_id).await?;
    Ok(no_content_response())
}
