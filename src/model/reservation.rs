use sqlx::{FromRow};
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use crate::utils;


#[derive(Deserialize)]
pub struct ReservationPost {
  pub resource_id: i32,
  pub user_id: i32,
  pub start: DateTime<Local>,
  pub end: DateTime<Local>,
  pub description: Option<String>,
  pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct ReservationPut {
  pub resource_id: Option<i32>,
  pub user_id: Option<i32>,
  pub start: Option<DateTime<Local>>,
  pub end: Option<DateTime<Local>>,
  pub description: Option<String>,
  pub password: Option<String>,
  pub new_password: Option<String>,
}

#[derive(Serialize)]
pub struct ReservationReturn {
  pub id: i32,
  pub resource_id: i32,
  pub resource_name: String,
  pub user_id: i32,
  pub user_name: String,
  pub start: DateTime<Local>,
  pub end: DateTime<Local>,
  pub description: Option<String>,
}

#[derive(FromRow, Serialize)]
pub struct ReservationDB {
  pub id: i32,
  pub resource_id: i32,
  pub resource_name: String,
  pub user_id: i32,
  pub user_name: String,
  pub start: i64,
  pub end: i64,
  pub description: Option<String>,
  pub passhash: Option<String>,
}

#[derive(Deserialize)]
pub struct Filter {
    pub from: Option<DateTime<Local>>,
    pub until: Option<DateTime<Local>>,
    pub user_id: Option<i32>,
    pub resource_id: Option<i32>,
}

#[derive(Serialize)]
pub struct FullCalendarEvent {
    pub id: i32,
    pub title: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub description: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct FullCalendarFilter {
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
    pub resource_id: Option<i32>,
}

impl ReservationPost {
  pub fn is_valid(self: &Self) -> bool {
    if self.end > self.start {true} else {false}
  }
}

impl From<ReservationDB> for ReservationReturn {
  fn from(db: ReservationDB) -> Self {
    let start = Local.timestamp(db.start, 0);
    let end = Local.timestamp(db.end, 0);
    ReservationReturn {
      id: db.id, 
      resource_id: db.resource_id,
      resource_name: db.resource_name,
      user_id: db.user_id,
      user_name: db.user_name,
      start,
      end,
      description: db.description,         
    }
  }
}

impl From<ReservationPost> for ReservationDB {
  fn from(reservation: ReservationPost) -> Self {
    let start = reservation.start.timestamp();
    let end = reservation.end.timestamp();
    ReservationDB {
      id: 0,
      resource_id: reservation.resource_id,
      resource_name: "".to_string(),
      user_id: reservation.user_id,
      user_name: "".to_string(),
      start,
      end,
      description: reservation.description,
      passhash: Some(utils::hash_anyway(&reservation.password)),
    }
  }
}

impl From<ReservationReturn> for FullCalendarEvent {
  fn from(reservation: ReservationReturn) -> Self {
    let mut title = reservation.user_name;
    if let Some(description) = &reservation.description {
      if !description.is_empty() {
        let note = format!(" ({})", &description);
        title += &note;
      }
    }
    FullCalendarEvent {
      id: reservation.id,
      title: title,
      start: reservation.start,
      end: reservation.end,
      description: reservation.description,
    }
  }
}

impl From<FullCalendarFilter> for Filter {
  fn from(f: FullCalendarFilter) -> Self {
    Filter {
      from: f.start,
      until: f.end,
      user_id: None,
      resource_id: f.resource_id,
    }
  }
}