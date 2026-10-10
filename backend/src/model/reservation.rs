use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
#[derive(Deserialize)]
pub struct ReservationPost {
    pub resource_id: i32,
    pub user_id: i32,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub description: Option<String>,
    pub password: Option<String>,
}
#[derive(Deserialize)]
pub struct ReservationPut {
    pub resource_id: Option<i32>,
    pub user_id: Option<i32>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub password: Option<String>,
    pub new_password: Option<String>,
    pub version: Option<i32>,
}
#[derive(Serialize)]
pub struct ReservationReturn {
    pub id: i32,
    pub resource_id: i32,
    pub resource_name: String,
    pub user_id: i32,
    pub user_name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub version: i32,
}
#[derive(FromRow)]
pub struct ReservationDB {
    pub id: i32,
    pub resource_id: i32,
    pub resource_name: String,
    pub user_id: i32,
    pub user_name: String,
    pub start: i64,
    pub end: i64,
    pub description: Option<String>,
    pub created_at: i64,
    pub version: i32,
}
#[derive(Deserialize)]
pub struct Filter {
    pub from: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub user_id: Option<i32>,
    pub resource_id: Option<i32>,
}
#[derive(Serialize)]
pub struct FullCalendarEvent {
    pub id: i32,
    pub title: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub description: Option<String>,
    pub resource_id: i32,
}
#[derive(Deserialize)]
pub struct FullCalendarFilter {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub resource_id: Option<i32>,
    pub user_id: Option<i32>,
}
impl ReservationPost {
    pub fn is_valid(&self) -> bool {
        self.start < self.end
    }
}
impl TryFrom<ReservationDB> for ReservationReturn {
    type Error = sqlx::Error;
    fn try_from(db: ReservationDB) -> Result<Self, Self::Error> {
        let time = |v| {
            DateTime::from_timestamp(v, 0)
                .ok_or_else(|| sqlx::Error::Protocol("Invalid timestamp in database".into()))
        };
        Ok(Self {
            id: db.id,
            resource_id: db.resource_id,
            resource_name: db.resource_name,
            user_id: db.user_id,
            user_name: db.user_name,
            start: time(db.start)?,
            end: time(db.end)?,
            description: db.description,
            created_at: time(db.created_at)?,
            version: db.version,
        })
    }
}
impl From<ReservationReturn> for FullCalendarEvent {
    fn from(r: ReservationReturn) -> Self {
        Self {
            id: r.id,
            title: match &r.description {
                Some(note) if !note.is_empty() => format!("{} · {}", r.user_name, note),
                _ => r.user_name,
            },
            start: r.start,
            end: r.end,
            description: r.description,
            resource_id: r.resource_id,
        }
    }
}
impl From<FullCalendarFilter> for Filter {
    fn from(f: FullCalendarFilter) -> Self {
        Self {
            from: f.start,
            until: f.end,
            resource_id: f.resource_id,
            user_id: f.user_id,
        }
    }
}
