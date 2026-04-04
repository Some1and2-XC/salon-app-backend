use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::{
    AppState, middleware::auth::AuthenticatedUser, models::{
        appointment::Appointment,
        appointment_availability::{
            AppointmentAvailability, CreateAvailabilityRequest, QueryAvailabilityParams,
        },
        appointment_state::{self, AppointmentState, KnownState},
    }, routes::users::{forbidden, internal_error, not_found}
};

/// GET /availability — authenticated users query open slots. Admins view all available slots.
#[utoipa::path(
    get,
    path = "/availability",
    params(QueryAvailabilityParams),
    responses(
        (status = 200, description = "List of availability slots", body = Vec<AppointmentAvailability>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
pub async fn list_availability(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<QueryAvailabilityParams>,
) -> Result<Json<Vec<AppointmentAvailability>>, (StatusCode, Json<serde_json::Value>)> {

    let slots = sqlx::query_as!(
        AppointmentAvailability,
        r#"SELECT id, employee_id, start_time, end_time
           FROM appointment_availability
           WHERE ($1   IS NULL OR employee_id = $1)
             AND ($2 IS NULL OR start_time  >= $2)
             AND ($3 IS NULL OR end_time    <= $3)
           ORDER BY start_time"#,
        params.employee_id,
        params.from,
        params.to,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    if auth.is_admin {
        return Ok(Json(slots));
    }

    let appointments = sqlx::query_as!(
        Appointment,
        r#"SELECT uuid, user_uuid, task_id, employee_id, start_time, length,
                  appointment_state_id, date_created, last_modified
           FROM appointments
           WHERE ($1 IS NULL OR employee_id = $1)
             AND ($2 IS NULL OR appointment_state_id = $2)
             AND ($3 IS NULL OR start_time >= $3)
             AND ($4 IS NULL OR start_time <= $4)
           ORDER BY start_time"#,
        params.employee_id,
        params.state_id,
        params.from,
        params.to,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?
    .into_iter()
    .filter(|v| KnownState::requires_employee(v.appointment_state_id))
    .collect::<Vec<_>>()
    ;

    let result = slots
        .into_iter()
        .flat_map(|slot| {
            let mut overlapping = appointments
                .iter()
                .filter_map(|appt| {
                    let appt_end = appt.start_time + appt.length as i64;
                    if appt.start_time < slot.end_time && appt_end > slot.start_time {
                        Some((appt.start_time, appt_end))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if overlapping.is_empty() {
                return vec![slot.clone()];
            }

            overlapping.sort_by_key(|(start, _)| *start);

            let mut free_slots = Vec::new();
            let mut cursor = slot.start_time;

            for (appt_start, appt_end) in overlapping {
                let blocked_start = appt_start.max(slot.start_time);
                let blocked_end = appt_end.min(slot.end_time);

                if blocked_start > cursor {
                    free_slots.push(AppointmentAvailability {
                        id: slot.id,
                        employee_id: slot.employee_id.clone(),
                        start_time: cursor,
                        end_time: blocked_start,
                    });
                }

                cursor = cursor.max(blocked_end);
            }

            if cursor < slot.end_time {
                free_slots.push(AppointmentAvailability {
                    id: slot.id,
                    employee_id: slot.employee_id.clone(),
                    start_time: cursor,
                    end_time: slot.end_time,
                });
            }

            free_slots
        })
        .collect::<Vec<_>>();

    Ok(Json(result))
}

/// POST /availability — admin only.
#[utoipa::path(
    post,
    path = "/availability",
    request_body = CreateAvailabilityRequest,
    responses(
        (status = 201, description = "Availability slot created successfully", body = AppointmentAvailability),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
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
#[utoipa::path(
    delete,
    path = "/availability/{id}",
    params(
        ("id" = i32, Path, description = "Availability slot ID")
    ),
    responses(
        (status = 204, description = "Availability slot deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Availability slot not found"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_token" = []))
)]
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
