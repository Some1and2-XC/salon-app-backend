use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A validated phone number, always serialized as a string.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct Phone(pub String);

impl Phone {
    /// Creates a new Phone, returning an error if the number is invalid.
    pub fn new(number: impl Into<String>) -> Result<Self, PhoneError> {
        let number = number.into();
        let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() < 7 || digits.len() > 15 {
            return Err(PhoneError::InvalidLength(digits.len()));
        }
        Ok(Phone(number))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PhoneError {
    #[error("Phone number has invalid length: {0} digits (expected 7–15)")]
    InvalidLength(usize),
}

/// Mirrors the frontend `User` class.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    // pub uuid: Option<Uuid>,
    pub uuid: Option<String>,
    /// Stored as plain text in the DB; wrapped in Phone for validation at the API boundary.
    pub phone: Option<String>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    /// Unix timestamp (milliseconds).
    pub date_created: Option<i64>,
    /// Unix timestamp (milliseconds).
    pub last_modified: Option<i64>,
    pub admin: bool,
}

/// The subset of `User` fields a client is allowed to supply on creation / update.
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub phone: Option<String>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
