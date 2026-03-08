use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;

use crate::{
    middleware::auth::AuthenticatedUser,
    models::{
        employee::{CreateEmployeeRequest, Employee, UpdateEmployeeRequest},
        user::Phone,
    },
    routes::users::{bad_request, forbidden, internal_error, not_found},
    AppState,
};

/// GET /employees — authenticated users can list employees.
pub async fn list_employees(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Employee>>, (StatusCode, Json<serde_json::Value>)> {
    let employees = sqlx::query_as!(
        Employee,
        "SELECT id, first_name, last_name, phone, email, date_created, last_modified
         FROM employees ORDER BY last_name, first_name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(employees))
}

/// GET /employees/:id
pub async fn get_employee(
    _auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Employee>, (StatusCode, Json<serde_json::Value>)> {
    let emp = sqlx::query_as!(
        Employee,
        "SELECT id, first_name, last_name, phone, email, date_created, last_modified
         FROM employees WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Employee not found"))?;

    Ok(Json(emp))
}

/// POST /employees — admin only.
pub async fn create_employee(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateEmployeeRequest>,
) -> Result<(StatusCode, Json<Employee>), (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }
    // Validate phone at the API boundary.
    Phone::new(&req.phone).map_err(|e| bad_request(&e.to_string()))?;

    let now = Utc::now().timestamp_millis();
    let emp = sqlx::query_as!(
        Employee,
        r#"INSERT INTO employees (id, first_name, last_name, phone, email, date_created, last_modified)
           VALUES ($1, $2, $3, $4, $5, $6, $6)
           RETURNING id, first_name, last_name, phone, email, date_created, last_modified"#,
        req.id,
        req.first_name,
        req.last_name,
        req.phone,
        req.email,
        now,
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(emp)))
}

/// PATCH /employees/:id — admin only.
pub async fn update_employee(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateEmployeeRequest>,
) -> Result<Json<Employee>, (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }
    if let Some(ref phone) = req.phone {
        Phone::new(phone).map_err(|e| bad_request(&e.to_string()))?;
    }

    let now = Utc::now().timestamp_millis();
    let emp = sqlx::query_as!(
        Employee,
        r#"UPDATE employees
           SET first_name    = COALESCE($2, first_name),
               last_name     = COALESCE($3, last_name),
               phone         = COALESCE($4, phone),
               email         = COALESCE($5, email),
               last_modified = $6
           WHERE id = $1
           RETURNING id, first_name, last_name, phone, email, date_created, last_modified"#,
        id,
        req.first_name,
        req.last_name,
        req.phone,
        req.email,
        now,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| not_found("Employee not found"))?;

    Ok(Json(emp))
}

/// DELETE /employees/:id — admin only.
pub async fn delete_employee(
    auth: AuthenticatedUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if !auth.is_admin {
        return Err(forbidden());
    }

    let result = sqlx::query!("DELETE FROM employees WHERE id = $1", id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err(not_found("Employee not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
