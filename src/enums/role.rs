use async_graphql::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio_postgres::types::{FromSql, ToSql};

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Role {
    User,
    Manager,
    Admin,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Manager => write!(f, "Manager"),
            Role::Admin => write!(f, "Admin"),
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

impl Role {
    pub fn to_string(&self) -> String {
        match self {
            Role::User => "User".to_string(),
            Role::Manager => "Manager".to_string(),
            Role::Admin => "Admin".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Role {
        match s {
            "User" => Role::User,
            "Manager" => Role::Manager,
            "Admin" => Role::Admin,
            _ => Role::User,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Role::User => 0,
            Role::Manager => 1,
            Role::Admin => 2,
        }
    }

    pub fn from_i32(i: i32) -> Role {
        match i {
            0 => Role::User,
            1 => Role::Manager,
            2 => Role::Admin,
            _ => Role::User,
        }
    }
}

impl FromSql<'_> for Role {
    fn from_sql(
        _: &tokio_postgres::types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let s = String::from_utf8(raw.to_vec()).unwrap();
        Ok(Role::from_string(&s))
    }

    fn accepts(_: &tokio_postgres::types::Type) -> bool {
        true
    }
}

impl ToSql for Role {
    fn to_sql(
        &self,
        _: &tokio_postgres::types::Type,
        w: &mut bytes::BytesMut,
    ) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let s = self.to_string();
        w.extend_from_slice(s.as_bytes());
        Ok(tokio_postgres::types::IsNull::No)
    }

    fn accepts(_: &tokio_postgres::types::Type) -> bool {
        true
    }

    tokio_postgres::types::to_sql_checked!();
}
