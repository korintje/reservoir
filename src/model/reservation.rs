use sqlx::{FromRow};
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use crate::utils;


#[derive(Debug, Serialize, Deserialize)]
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

#[derive(FromRow, Serialize)]
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

#[derive(Deserialize)]
pub struct Filter {
    pub from: Option<DateTime<Local>>,
    pub until: Option<DateTime<Local>>,
    pub user_id: Option<i32>,
    pub resource_id: Option<i32>,
}

#[derive(Serialize)]
pub struct FullCalendarEvent {
    pub title: Option<String>,
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
    pub description: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct FullCalendarFilter {
    pub start: Option<DateTime<Local>>,
    pub end: Option<DateTime<Local>>,
    pub resource_id: Option<i32>,
}

impl From<ReservationDB> for Reservation {
  fn from(db: ReservationDB) -> Self {
    let start = db.start.map(|t| Local.timestamp(t, 0));
    let end = db.end.map(|t| Local.timestamp(t, 0));
    Reservation {
      id: db.id, 
      resource_id: db.resource_id,
      resource_name: db.resource_name,
      user_id: db.user_id,
      user_name: db.user_name,
      start,
      end,
      description: db.description,
      password: None,            
    }
  }
}

impl From<Reservation> for ReservationDB {
  fn from(reservation: Reservation) -> Self {
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
      passhash: Some(utils::hash(&reservation.password)),
    }
  }
}

impl From<Reservation> for FullCalendarEvent {
  fn from(reservation: Reservation) -> Self {
    FullCalendarEvent {
      title: reservation.user_name,
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