use axum::{Json, extract::State, http::StatusCode};

use crate::{AppState, models::task_category::TaskCategory, routes::users::internal_error};

/// GET /task-categories — Returns the categories for tasks (public).
#[utoipa::path(
    get,
    path = "/task-categories",
    responses(
        (status = 200, description = "List of all task categories", body = Vec<TaskCategory>),
        (status = 500, description = "Internal server error"),
    )
)]
pub async fn list_task_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<TaskCategory>>, (StatusCode, Json<serde_json::Value>)> {
    let tasks = sqlx::query_as!(
        TaskCategory,
        "SELECT id, name FROM task_categories"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    return Ok(Json(tasks));
}
