use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

use crate::{
    middleware::auth::{AuthenticatedUser},
    models::user::{CreateUserRequest, UpdateUserRequest, User},
    AppState,
};

/// GET /users/me — return the authenticated user's own profile.
pub async fn get_me(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT uuid, phone, email, first_name, last_name,
                  date_created, last_modified, admin
           FROM users WHERE uuid = $1"#,
        auth.uid
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("User not found"))?;

    Ok(Json(user))
}

/// POST /users — create a user record tied to the Firebase UID.
pub async fn create_user(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now().timestamp_millis();
    let uid = &auth.uid;

    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO users (uuid, phone, email, first_name, last_name, date_created, last_modified, admin)
           VALUES ($1, $2, $3, $4, $5, $6, $6, false)
           RETURNING uuid, phone, email, first_name, last_name, date_created, last_modified, admin"#,
        uid,
        req.phone,
        req.email,
        req.first_name,
        req.last_name,
        now,
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(user)))
}

/// PATCH /users/me — update the authenticated user's own profile.
pub async fn update_me(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now().timestamp_millis();
    let uid = &auth.uid;

    let user = sqlx::query_as!(
        User,
        r#"UPDATE users
           SET phone         = COALESCE($2, phone),
               email         = COALESCE($3, email),
               first_name    = COALESCE($4, first_name),
               last_name     = COALESCE($5, last_name),
               last_modified = $6
           WHERE uuid = $1
           RETURNING uuid, phone, email, first_name, last_name, date_created, last_modified, admin"#,
        uid,
        req.phone,
        req.email,
        req.first_name,
        req.last_name,
        now,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("User not found"))?;

    Ok(Json(user))
}

/// GET /users/:uuid — admin-only: fetch any user.
pub async fn get_user_by_id(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }

    let user = sqlx::query_as!(
        User,
        r#"SELECT uuid, phone, email, first_name, last_name,
                  date_created, last_modified, admin
           FROM users WHERE uuid = $1"#,
        uuid,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("User not found"))?;

    Ok(Json(user))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn internal_error(e: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    tracing::error!("DB error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Internal server error" })),
    )
}

pub fn not_found(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": msg })),
    )
}

pub fn bad_request(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg })),
    )
}

pub fn forbidden() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Forbidden" })),
    )
}
