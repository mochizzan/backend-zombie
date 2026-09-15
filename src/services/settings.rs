use std::collections::HashMap;

use sqlx::MySqlPool;

use crate::errors::AppError;
use crate::models::user_settings::UserSettings;

pub async fn get_settings(
    pool: &MySqlPool,
    player_id: &str,
) -> Result<HashMap<String, u8>, AppError> {
    let user = crate::services::user::get_user(pool, player_id).await?;
    let rows = sqlx::query_as::<_, UserSettings>(
        "SELECT * FROM user_settings WHERE user_id = ? AND deleted_at IS NULL",
    )
    .bind(user.id)
    .fetch_all(pool)
    .await?;

    let map = rows
        .into_iter()
        .map(|r| (r.setting_key, r.setting_value))
        .collect();
    Ok(map)
}

pub async fn update_settings(
    pool: &MySqlPool,
    player_id: &str,
    settings: HashMap<String, u8>,
) -> Result<HashMap<String, u8>, AppError> {
    let user = crate::services::user::get_user(pool, player_id).await?;

    for (key, value) in &settings {
        sqlx::query(
            "INSERT INTO user_settings (user_id, setting_key, setting_value) VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), deleted_at = NULL",
        )
        .bind(user.id)
        .bind(key)
        .bind(*value)
        .execute(pool)
        .await?;
    }

    get_settings(pool, player_id).await
}

pub async fn update_single(
    pool: &MySqlPool,
    player_id: &str,
    key: &str,
    value: u8,
) -> Result<(), AppError> {
    let user = crate::services::user::get_user(pool, player_id).await?;
    sqlx::query(
        "INSERT INTO user_settings (user_id, setting_key, setting_value) VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), deleted_at = NULL",
    )
    .bind(user.id)
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_setting(
    pool: &MySqlPool,
    player_id: &str,
    key: &str,
) -> Result<(), AppError> {
    let user = crate::services::user::get_user(pool, player_id).await?;
    let result = sqlx::query(
        "UPDATE user_settings SET deleted_at = NOW() WHERE user_id = ? AND setting_key = ? AND deleted_at IS NULL",
    )
    .bind(user.id)
    .bind(key)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Setting '{}' not found",
            key
        )));
    }
    Ok(())
}
