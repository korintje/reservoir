use sqlx::{FromRow};
use serde::{Deserialize, Serialize};

pub mod reservation;
pub use reservation::*;

// fn default_zero() -> i32 { 0 }
// fn default_true() -> bool { true }

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
  pub id: Option<i32>,
  pub user_name: Option<String>,
  pub active: Option<bool>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Resource {
  pub id: Option<i32>,
  pub resource_name: Option<String>,
  pub active: Option<bool>,
  pub capacity: Option<i32>,
  pub custom_field: Option<String>,
}

/*
#[derive(FromRow, Serialize, Deserialize)]
pub struct Resource {
  #[serde(default = "default_zero")]
  pub id: i32,
  pub resource_name: String,
  #[serde(default = "default_true")]
  pub active: bool,
}
*/

#[derive(FromRow, Serialize)]
pub struct PassHash {
    pub passhash: Option<String>,
}

#[derive(Deserialize)]
pub struct PassWord {
    pub password: Option<String>,
}

#[derive(FromRow, Serialize)]
pub struct TableCount {
    pub count: i32,
}

/*
#[derive(FromRow, Serialize, Deserialize)]
pub struct Label {
  pub id: Option<i32>,
  pub reservation_id: Option<i32>,
  pub user_id: Option<i32>,
  pub category: Option<i32>,
  pub description: Option<i32>,
  pub passhash: Option<String>,
}
*/