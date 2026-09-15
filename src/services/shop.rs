use sqlx::MySqlPool;
use crate::errors::AppError;
use crate::models::shop_item::{ShopItem, CreateShopItem, UpdateShopItem};
use base64::{Engine, engine::general_purpose};

pub async fn list_items(
    pool: &MySqlPool,
    role: Option<&str>,
    category: Option<&str>,
    cursor: Option<&str>,
    limit: i64,
) -> Result<(Vec<ShopItem>, Option<String>, bool), AppError> {
    let last_id = cursor
        .and_then(|c| general_purpose::STANDARD.decode(c).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .and_then(|s| s.parse::<i64>().ok());

    let fetch_limit = limit + 1;

    let mut where_clauses = vec!["deleted_at IS NULL".to_string()];
    let mut binds: Vec<String> = Vec::new();

    if let Some(r) = role {
        where_clauses.push("role = ?".to_string());
        binds.push(r.to_string());
    }
    if let Some(c) = category {
        where_clauses.push("category = ?".to_string());
        binds.push(c.to_string());
    }
    if last_id.is_some() {
        where_clauses.push("id > ?".to_string());
    }

    let sql = format!(
        "SELECT * FROM shop_items WHERE {} ORDER BY id ASC LIMIT {}",
        where_clauses.join(" AND "),
        fetch_limit
    );

    let mut q = sqlx::query_as::<_, ShopItem>(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    if let Some(id) = last_id {
        q = q.bind(id);
    }

    let mut items = q.fetch_all(pool).await?;
    let has_more = items.len() as i64 > limit;
    if has_more {
        items.pop();
    }
    let next_cursor = items
        .last()
        .map(|i| general_purpose::STANDARD.encode(i.id.to_string()));

    Ok((items, next_cursor, has_more))
}

pub async fn get_item(pool: &MySqlPool, id: i64) -> Result<ShopItem, AppError> {
    let item = sqlx::query_as::<_, ShopItem>(
        "SELECT * FROM shop_items WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    item.ok_or_else(|| AppError::NotFound(format!("Shop item {} not found", id)))
}

pub async fn create_item(pool: &MySqlPool, data: CreateShopItem) -> Result<ShopItem, AppError> {
    let discount = data.discount_percent.unwrap_or(0);
    let discounted_price = data.price * (100 - discount as i64) / 100;

    let result = sqlx::query(
        "INSERT INTO shop_items (role, category, name, description, price, discount_percent, discounted_price) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&data.role)
    .bind(&data.category)
    .bind(&data.name)
    .bind(&data.description)
    .bind(data.price)
    .bind(discount)
    .bind(discounted_price)
    .execute(pool)
    .await?;

    get_item(pool, result.last_insert_id() as i64).await
}

pub async fn update_item(
    pool: &MySqlPool,
    id: i64,
    data: UpdateShopItem,
) -> Result<ShopItem, AppError> {
    let existing = get_item(pool, id).await?;

    let role = data.role.unwrap_or(existing.role);
    let category = data.category.unwrap_or(existing.category);
    let name = data.name.unwrap_or(existing.name);
    let description = data.description.or(existing.description);
    let price = data.price.unwrap_or(existing.price);
    let discount = data.discount_percent.or(existing.discount_percent).unwrap_or(0);
    let discounted_price = price * (100 - discount as i64) / 100;
    let is_active = data.is_active.unwrap_or(existing.is_active);

    sqlx::query(
        "UPDATE shop_items SET role=?, category=?, name=?, description=?, price=?, discount_percent=?, discounted_price=?, is_active=? WHERE id=? AND deleted_at IS NULL",
    )
    .bind(&role)
    .bind(&category)
    .bind(&name)
    .bind(&description)
    .bind(price)
    .bind(discount)
    .bind(discounted_price)
    .bind(is_active)
    .bind(id)
    .execute(pool)
    .await?;

    get_item(pool, id).await
}

pub async fn delete_item(pool: &MySqlPool, id: i64) -> Result<(), AppError> {
    let result =
        sqlx::query("UPDATE shop_items SET deleted_at = NOW() WHERE id = ? AND deleted_at IS NULL")
            .bind(id)
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Shop item {} not found", id)));
    }
    Ok(())
}
