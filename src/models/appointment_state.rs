use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Mirrors the frontend `AppointmentState` class and its constants.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct AppointmentState {
    pub id: i64,
    pub name: String,
}

/// Strongly-typed enum for the well-known appointment states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownState {
    Unconfirmed = 0,
    Accepted = 1,
    Confirmed = 2,
    Cancelled = 3,
    Completed = 4,
}

impl KnownState {
    pub fn id(self) -> i64 {
        self as i64
    }

    /// Returns `true` when an `employee_id` is required (mirrors frontend `validate()`).
    pub fn requires_employee(id: i64) -> bool {
        matches!(
            id,
            x if x == KnownState::Accepted.id()
              || x == KnownState::Confirmed.id()
              || x == KnownState::Completed.id()
        )
    }
}
