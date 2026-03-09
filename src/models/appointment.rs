use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::models::appointment_state::KnownState;

/// Mirrors the frontend `Appointment` class.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Appointment {
    pub uuid: Option<String>,
    pub user_uuid: String,
    pub task_id: i64,
    /// `None` is only valid while the appointment is Unconfirmed.
    pub employee_id: Option<String>,
    /// Unix timestamp (milliseconds).
    pub start_time: i64,
    /// Duration in seconds.
    pub length: i64,
    pub appointment_state_id: i64,
    /// Unix timestamp (milliseconds).
    pub date_created: Option<i64>,
    /// Unix timestamp (milliseconds).
    pub last_modified: Option<i64>,
}

impl Appointment {
    /// Mirrors the frontend `validate()` method.
    /// Returns `true` when the appointment is in a consistent state.
    pub fn validate(&self) -> bool {
        if KnownState::requires_employee(self.appointment_state_id) && self.employee_id.is_none() {
            return false;
        }
        true
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAppointmentRequest {
    pub task_id: i64,
    pub employee_id: Option<String>,
    pub start_time: i64,
    pub length: i64,
    /// Defaults to `0` (Unconfirmed) if omitted.
    pub appointment_state_id: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAppointmentRequest {
    pub employee_id: Option<String>,
    pub start_time: Option<i64>,
    pub length: Option<i64>,
    pub appointment_state_id: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct QueryAppointmentsParams {
    pub user_uuid: Option<String>,
    pub employee_id: Option<String>,
    pub state_id: Option<i64>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}
