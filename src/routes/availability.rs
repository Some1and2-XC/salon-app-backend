use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::{
    middleware::auth::AuthenticatedUser,
    models::appointment_availability::{
        AppointmentAvailability, CreateAvailabilityRequest, QueryAvailabilityParams,
    },
    routes::users::{forbidden, internal_error, not_found},
    AppState,
};

/// GET /availability — authenticated users query open slots.
pub async fn list_availability(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<QueryAvailabilityParams>,
) -> Result<Json<Vec<AppointmentAvailability>>, (StatusCode, Json<serde_json::Value>)> {
    let slots = sqlx::query_as!(
        AppointmentAvailability,
        r#"SELECT id, employee_id, start_time, end_time
           FROM appointment_availability
           WHERE ($1::text   IS NULL OR employee_id = $1)
             AND ($2::bigint IS NULL OR start_time  >= $2)
             AND ($3::bigint IS NULL OR end_time    <= $3)
           ORDER BY start_time"#,
        params.employee_id,
        params.from,
        params.to,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(slots))
}

/// POST /availability — admin only.
pub async fn create_availability(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateAvailabilityRequest>,
) -> Result<(StatusCode, Json<AppointmentAvailability>), (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }

    let slot = sqlx::query_as!(
        AppointmentAvailability,
        r#"INSERT INTO appointment_availability (employee_id, start_time, end_time)
           VALUES ($1, $2, $3)
           RETURNING id, employee_id, start_time, end_time"#,
        req.employee_id,
        req.start_time,
        req.end_time,
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(slot)))
}

/// DELETE /availability/:id — admin only.
pub async fn delete_availability(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }

    let result = sqlx::query!("DELETE FROM appointment_availability WHERE id = $1", id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Availability slot not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
