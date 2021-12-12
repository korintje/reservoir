use sqlx::{FromRow};
use serde::{Deserialize, Serialize};

pub mod reservation;
pub use reservation::*;

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
  pub id: i32,
  pub user_name: String,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Resource {
  pub id: i32,
  pub resource_name: String,
}

#[derive(FromRow, Serialize)]
pub struct PassHash {
    pub passhash: Option<String>,
}

#[derive(FromRow, Serialize)]
pub struct TableCount {
    pub count: i32,
}