use sqlx::{FromRow};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct FullCalendarPeriod {
    pub resource: i32,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

#[derive(FromRow, Serialize)]
pub struct FullCalendarEvent {
    pub id: i32,
    pub title: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub description: String,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct ReservationInfo {
    pub resource_id: i32,
    pub user_id: i32,
    pub start_datetime: DateTime<Local>,
    pub end_datetime: DateTime<Local>,
    pub description: String,
    pub password: String,
}

#[derive(FromRow, Serialize)]
pub struct ReservationOut {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct DeleteInfo {
    pub id: i32,
    pub password: String,
}

#[derive(FromRow)]
pub struct StoredPass {
    pub id: i32,
    pub passhash: String,
}

#[derive(Serialize)]
pub struct MyError {
    pub code: i32,
    pub message: String,
}