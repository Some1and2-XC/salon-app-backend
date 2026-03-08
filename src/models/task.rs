use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Mirrors the frontend `Task` class.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: i64,
    pub name: String,
    /// Duration in seconds that should be booked for this task.
    pub time_for_booking: i64,
    /// Unix timestamp (milliseconds).
    pub date_created: Option<i64>,
    /// Unix timestamp (milliseconds).
    pub last_modified: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub time_for_booking: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub name: Option<String>,
    pub time_for_booking: Option<i64>,
}
