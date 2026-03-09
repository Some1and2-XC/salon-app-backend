use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

use crate::{
    middleware::auth::AuthenticatedUser,
    models::task::{CreateTaskRequest, Task, UpdateTaskRequest},
    routes::users::{bad_request, forbidden, internal_error, not_found},
    AppState,
};

/// GET /tasks — list all tasks (public).
pub async fn list_tasks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Task>>, (StatusCode, Json<serde_json::Value>)> {
    let tasks = sqlx::query_as!(
        Task,
        "SELECT id, name, time_for_booking, price_cad_cent, task_category_id, date_created, last_modified FROM tasks ORDER BY id"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(tasks))
}

/// GET /tasks/:id
pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Task>, (StatusCode, Json<serde_json::Value>)> {
    let task = sqlx::query_as!(
        Task,
        "SELECT id, name, time_for_booking, price_cad_cent, task_category_id, date_created, last_modified FROM tasks WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Task not found"))?;

    Ok(Json(task))
}

/// POST /tasks — admin only.
pub async fn create_task(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }
    if req.time_for_booking <= 0 {
        return Err(bad_request("time_for_booking must be positive"));
    }

    let now = Utc::now().timestamp_millis();
    let task = sqlx::query_as!(
        Task,
        r#"INSERT INTO tasks (name, time_for_booking, price_cad_cent, task_category_id, date_created, last_modified)
           VALUES ($1, $2, $3, $4, $5, $5)
           RETURNING id, name, time_for_booking, price_cad_cent, task_category_id, date_created, last_modified"#,
        req.name,
        req.time_for_booking,
        req.price_cad_cent,
        req.task_category_id,
        now,
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(task)))
}

/// PATCH /tasks/:id — admin only.
pub async fn update_task(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }

    let now = Utc::now().timestamp_millis();
    let task = sqlx::query_as!(
        Task,
        r#"UPDATE tasks
           SET name             = COALESCE($2, name),
               time_for_booking = COALESCE($3, time_for_booking),
               price_cad_cent   = COALESCE($4, price_cad_cent),
               task_category_id = COALESCE($5, task_category_id),
               last_modified    = $6
           WHERE id = $1
           RETURNING id, name, time_for_booking, price_cad_cent, task_category_id, date_created, last_modified"#,
        id,
        req.name,
        req.time_for_booking,
        req.price_cad_cent,
        req.task_category_id,
        now,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Task not found"))?;

    Ok(Json(task))
}

/// DELETE /tasks/:id — admin only.
pub async fn delete_task(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }

    let result = sqlx::query!("DELETE FROM tasks WHERE id = $1", id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Task not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
