use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Mirrors the frontend `Employee` class.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Employee {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    /// Stored as plain text; validated as `Phone` at the API boundary.
    pub phone: String,
    pub email: String,
    /// Unix timestamp (milliseconds).
    pub date_created: Option<i64>,
    /// Unix timestamp (milliseconds).
    pub last_modified: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEmployeeRequest {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEmployeeRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}
