use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Mirrors the frontend `AppointmentAvailability` class.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppointmentAvailability {
    pub id: i32,
    /// `None` means the slot is open to any employee.
    pub employee_id: Option<String>,
    /// Unix timestamp (milliseconds).
    pub start_time: i64,
    /// Unix timestamp (milliseconds).
    pub end_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateAvailabilityRequest {
    pub employee_id: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct QueryAvailabilityParams {
    /// Filter by employee.
    pub employee_id: Option<String>,
    /// Only return slots that start on or after this timestamp (ms).
    pub from: Option<i64>,
    /// Only return slots that end on or before this timestamp (ms).
    pub to: Option<i64>,
}
