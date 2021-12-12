use sqlx::{FromRow};
use serde::{Deserialize, Serialize};

pub mod reservation;
pub use reservation::*;

fn default_zero() -> i32 { 0 }

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
  #[serde(default = "default_zero")]
  pub id: i32,
  pub user_name: String,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Resource {
  #[serde(default = "default_zero")]
  pub id: i32,
  pub resource_name: String,
}

#[derive(FromRow, Serialize)]
pub struct PassHash {
    pub value: Option<String>,
}

#[derive(Deserialize)]
pub struct PassWord {
    pub value: Option<String>,
}

#[derive(FromRow, Serialize)]
pub struct TableCount {
    pub count: i32,
}