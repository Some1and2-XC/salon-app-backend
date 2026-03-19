use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::{Timestamp, Uuid};

use crate::{
    middleware::auth::AuthenticatedUser,
    models::appointment::{
        Appointment, CreateAppointmentRequest, QueryAppointmentsParams, UpdateAppointmentRequest,
    },
    models::appointment_state::KnownState,
    routes::users::{bad_request, forbidden, internal_error, not_found},
    AppState,
};

/// GET /appointments — admins see all; users see only their own.
#[utoipa::path(
    get,
    path = "/appointments",
    params(QueryAppointmentsParams),
    responses(
        (status = 200, description = "List of appointments", body = Vec<Appointment>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
pub async fn list_appointments(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<QueryAppointmentsParams>,
) -> Result<Json<Vec<Appointment>>, (StatusCode, Json<serde_json::Value>)> {
    // Non-admins are restricted to their own appointments.
    let user_filter = if auth.is_admin {
        params.user_uuid
    } else {
        Some(auth.uid.clone())
    };

    let appointments = sqlx::query_as!(
        Appointment,
        r#"SELECT uuid, user_uuid, task_id, employee_id, start_time, length,
                  appointment_state_id, date_created, last_modified
           FROM appointments
           WHERE ($1 IS NULL OR user_uuid = $1)
             AND ($2 IS NULL OR employee_id = $2)
             AND ($3 IS NULL OR appointment_state_id = $3)
             AND ($4 IS NULL OR start_time >= $4)
             AND ($5 IS NULL OR start_time <= $5)
           ORDER BY start_time"#,
        user_filter,
        params.employee_id,
        params.state_id,
        params.from,
        params.to,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(appointments))
}

/// GET /appointments/:uuid
#[utoipa::path(
    get,
    path = "/appointments/{uuid}",
    params(
        ("uuid" = String, Path, description = "Appointment UUID")
    ),
    responses(
        (status = 200, description = "Appointment found", body = Appointment),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Appointment not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
pub async fn get_appointment(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<Json<Appointment>, (StatusCode, Json<serde_json::Value>)> {
    let appt = sqlx::query_as!(
        Appointment,
        r#"SELECT uuid, user_uuid, task_id, employee_id, start_time, length,
                  appointment_state_id, date_created, last_modified
           FROM appointments WHERE uuid = $1"#,
        uuid
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Appointment not found"))?;

    // Non-admins may only view their own appointments.
    if !auth.is_admin && appt.user_uuid != auth.uid {
        return Err(forbidden());
    }

    Ok(Json(appt))
}

/// POST /appointments — authenticated users create appointments for themselves.
#[utoipa::path(
    post,
    path = "/appointments",
    request_body = CreateAppointmentRequest,
    responses(
        (status = 201, description = "Appointment created successfully", body = Appointment),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
pub async fn create_appointment(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateAppointmentRequest>,
) -> Result<(StatusCode, Json<Appointment>), (StatusCode, Json<serde_json::Value>)> {
    let state_id = req.appointment_state_id.unwrap_or(KnownState::Unconfirmed.id());

    // Run the same validation logic as the frontend.
    if KnownState::requires_employee(state_id) && req.employee_id.is_none() {
        return Err(bad_request(
            "employee_id is required for Accepted/Confirmed/Completed appointments",
        ));
    }

    let now = Utc::now().timestamp_millis();
    let new_uuid = Uuid::now_v7().to_string();

    let appt = sqlx::query_as!(
        Appointment,
        r#"INSERT INTO appointments
               (uuid, user_uuid, task_id, employee_id, start_time, length,
                appointment_state_id, date_created, last_modified)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
           RETURNING uuid, user_uuid, task_id, employee_id, start_time, length,
                     appointment_state_id, date_created, last_modified"#,
        new_uuid,
        auth.uid,
        req.task_id,
        req.employee_id,
        req.start_time,
        req.length,
        state_id,
        now,
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(appt)))
}

/// PATCH /appointments/:uuid — users can update their own; admins can update any.
#[utoipa::path(
    patch,
    path = "/appointments/{uuid}",
    params(
        ("uuid" = String, Path, description = "Appointment UUID")
    ),
    request_body = UpdateAppointmentRequest,
    responses(
        (status = 200, description = "Appointment updated successfully", body = Appointment),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Appointment not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
pub async fn update_appointment(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
    Json(req): Json<UpdateAppointmentRequest>,
) -> Result<Json<Appointment>, (StatusCode, Json<serde_json::Value>)> {
    // Fetch existing to enforce ownership.
    let existing = sqlx::query_as!(
        Appointment,
        r#"SELECT uuid, user_uuid, task_id, employee_id, start_time, length,
                  appointment_state_id, date_created, last_modified
           FROM appointments WHERE uuid = $1"#,
        uuid
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Appointment not found"))?;

    if !auth.is_admin && existing.user_uuid != auth.uid {
        return Err(forbidden());
    }

    // Determine the new state and employee after merge.
    let new_state_id = req.appointment_state_id.unwrap_or(existing.appointment_state_id);
    let new_employee_id = req.employee_id.as_ref().or(existing.employee_id.as_ref());

    if KnownState::requires_employee(new_state_id) && new_employee_id.is_none() {
        return Err(bad_request(
            "employee_id is required for Accepted/Confirmed/Completed appointments",
        ));
    }

    let now = Utc::now().timestamp_millis();
    let appt = sqlx::query_as!(
        Appointment,
        r#"UPDATE appointments
           SET employee_id           = COALESCE($2, employee_id),
               start_time            = COALESCE($3, start_time),
               length                = COALESCE($4, length),
               appointment_state_id  = COALESCE($5, appointment_state_id),
               last_modified         = $6
           WHERE uuid = $1
           RETURNING uuid, user_uuid, task_id, employee_id, start_time, length,
                     appointment_state_id, date_created, last_modified"#,
        uuid,
        req.employee_id,
        req.start_time,
        req.length,
        req.appointment_state_id,
        now,
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(appt))
}

/// DELETE /appointments/:uuid — users cancel their own; admins delete any.
#[utoipa::path(
    delete,
    path = "/appointments/{uuid}",
    params(
        ("uuid" = String, Path, description = "Appointment UUID")
    ),
    responses(
        (status = 204, description = "Appointment deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Appointment not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
pub async fn delete_appointment(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(uuid): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let existing = sqlx::query!(
        "SELECT user_uuid FROM appointments WHERE uuid = $1",
        uuid
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Appointment not found"))?;

    if !auth.is_admin && existing.user_uuid != auth.uid {
        return Err(forbidden());
    }

    sqlx::query!("DELETE FROM appointments WHERE uuid = $1", uuid)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}
