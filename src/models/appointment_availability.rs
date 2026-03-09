use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

/// Mirrors the frontend `AppointmentAvailability` class.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AppointmentAvailability {
    pub id: i64,
    /// `None` means the slot is open to any employee.
    pub employee_id: Option<String>,
    /// Unix timestamp (milliseconds).
    pub start_time: i64,
    /// Unix timestamp (milliseconds).
    pub end_time: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAvailabilityRequest {
    pub employee_id: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct QueryAvailabilityParams {
    /// Filter by employee.
    pub employee_id: Option<String>,
    /// Only return slots that start on or after this timestamp (ms).
    pub from: Option<i64>,
    /// Only return slots that end on or before this timestamp (ms).
    pub to: Option<i64>,
}
