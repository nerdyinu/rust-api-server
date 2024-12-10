use crate::{
    error::AppError,
    models::{CreateItem, Item, UpdateItem},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_items(State(pool): State<PgPool>) -> Result<Json<Vec<Item>>, AppError> {
    let items = sqlx::query_as!(
        Item,
        r#"
        SELECT 
            id,
            name,
            description,
            created_at,
            updated_at
        FROM items
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("Internal Server Error")))?;

    Ok(Json(items))
}

pub async fn create_item(
    State(pool): State<PgPool>,
    Json(input): Json<CreateItem>,
) -> Result<Json<Item>, AppError> {
    let item = sqlx::query_as!(
        Item,
        r#"
        INSERT INTO items (id, name, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, description, created_at, updated_at
        "#,
        Uuid::new_v4(),
        input.name,
        input.description,
        Utc::now(),
        Utc::now()
    )
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(item))
}

pub async fn get_item(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Item>, AppError> {
    let item = sqlx::query_as!(
        Item,
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM items
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("Item not found: {}", id)))?;

    Ok(Json(item))
}

pub async fn update_item(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateItem>,
) -> Result<Json<Item>, AppError> {
    // First check if item exists
    let current_item = sqlx::query_as!(
        Item,
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM items
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("Item not found: {}", id)))?;

    // Update the item
    let item = sqlx::query_as!(
        Item,
        r#"
        UPDATE items
        SET 
            name = $1,
            description = $2,
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, name, description, created_at, updated_at
        "#,
        input.name.unwrap_or(current_item.name),
        input.description.or(current_item.description), // Fixed this line
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(item))
}

pub async fn delete_item(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM items
        WHERE id = $1
        RETURNING id
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(AppError::Database)?;

    if result.is_none() {
        return Err(AppError::NotFound(format!("Item not found: {}", id)));
    }

    Ok(Json(json!({
        "message": "Item deleted successfully",
        "id": id
    })))
}
