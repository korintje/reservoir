use sqlx::{FromRow};
use chrono::{DateTime, Local, Utc, TimeZone};
use serde::{Deserialize, Serialize};

fn default_id() -> i32 { 0 }

#[derive(FromRow, Serialize, Deserialize)]
pub struct User {
    #[serde(default = "default_id")]
    pub id: i32,
    pub user_name: String,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct Resource {
    #[serde(default = "default_id")]
    pub id: i32,
    pub resource_name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Reservation {
    pub id: Option<i32>,
    pub resource_id: Option<i32>,
    pub resource_name: Option<String>,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
    pub description: Option<String>,
    pub password: Option<String>,
}

impl Reservation {
    pub fn from_db(db: ReservationDB) -> Reservation {
        let start = db.start.map(|t| Local.timestamp(t, 0));
        let end = db.end.map(|t| Local.timestamp(t, 0));
        Reservation{
            id: db.id, 
            resource_id: db.resource_id,
            resource_name: db.resource_name,
            user_id: db.user_id,
            user_name: db.user_name,
            start,
            end,
            description: db.description,
            password: db.passhash,
        }
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Request {
    pub resource_id: i32,
    pub user_id: i32,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub description: String,
    pub password: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ReservationDB {
    pub id: Option<i32>,
    pub resource_id: Option<i32>,
    pub resource_name: Option<String>,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub description: Option<String>,
    pub passhash: Option<String>,
}

impl ReservationDB {
    pub fn from_web(reservation: Reservation) -> ReservationDB {
        let start = reservation.start.map(|t| t.timestamp());
        let end = reservation.end.map(|t| t.timestamp());
        ReservationDB {
            id: reservation.id,
            resource_id: reservation.resource_id,
            resource_name: reservation.resource_name,
            user_id: reservation.user_id,
            user_name: reservation.user_name,
            start,
            end,
            description: reservation.description,
            passhash: reservation.password,
        }
    }
}

#[derive(Deserialize)]
pub struct Period {
    pub from: DateTime<Local>,
    pub until: DateTime<Local>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct PassHash {
    pub passhash: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct TableCount {
    pub count: i32,
}

#[derive(Serialize)]
pub struct MyError {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize)]
pub struct MySuccess {
    pub code: i32,
    pub message: String,
}

/////////////////////////////////////////////////

/*
#[derive(Serialize, Deserialize)]
pub struct FullCalendarPeriod {
    pub resource: i32,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

#[derive(FromRow, Serialize, Deserialize)]
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

#[derive(FromRow, Serialize)]
pub struct ID {
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
*/