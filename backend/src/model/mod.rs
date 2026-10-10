use serde::{Deserialize, Serialize};
use sqlx::FromRow;
pub mod reservation;
pub use reservation::*;
#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Option<i32>,
    pub user_name: Option<String>,
    pub active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}
#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct Resource {
    pub id: Option<i32>,
    pub resource_name: Option<String>,
    pub active: Option<bool>,
    pub capacity: Option<i32>,
    pub custom_field: Option<String>,
    #[serde(default)]
    pub slack_channel_id: Option<String>,
    #[serde(default)]
    pub slack_url: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}
#[derive(FromRow)]
pub struct PassHash {
    pub passhash: Option<String>,
    pub version: i32,
}
#[derive(Deserialize)]
pub struct PassWord {
    pub password: Option<String>,
    pub version: Option<i32>,
}
#[derive(FromRow, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub lab_name: String,
    pub advance_days: i32,
    pub slot_minutes: i32,
    pub contact_text: String,
    pub notification_text: String,
}
#[derive(Serialize)]
pub struct MutationResult {
    pub id: i64,
    pub message: String,
    pub notification_required: bool,
    pub notification_reason: Option<String>,
    pub slack_url: Option<String>,
}
