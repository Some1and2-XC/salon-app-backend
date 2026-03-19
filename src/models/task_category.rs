use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// A category associated with a given task..
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TaskCategory {
    pub id: i64,
    pub name: String,
}

/// Strongly-typed enum for the well-known Task Categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownCategory {
    Manicure = 0,
    Pedicure = 1,
    Extension = 2,
    Kids = 3,
    AddOnService = 4,
    SpecialPackage = 5,
}

impl KnownCategory {

    pub fn id(self) -> i64 {
        self as i64
    }

    pub fn to_str(&self) -> &str {
        return match self {
            Self::Manicure => "Manicure",
            Self::Pedicure => "Pedicure",
            Self::Extension => "Extension",
            Self::Kids => "Kids",
            Self::AddOnService => "Add-On Service",
            Self::SpecialPackage => "Special Package",
        };
    }

}
