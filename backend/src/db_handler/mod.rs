use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{str::FromStr, sync::Arc, time::Duration};
pub mod reservation;
pub mod resource;
#[cfg(test)]
mod tests;
pub mod user;
pub struct DataAccessor {
    pub pool_ref: Arc<sqlx::SqlitePool>,
}
impl DataAccessor {
    pub async fn connect(path: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(path)?
            .create_if_missing(true)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(10));
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        Ok(Self {
            pool_ref: Arc::new(pool),
        })
    }
    #[cfg(test)]
    pub async fn new(path: &str) -> Self {
        Self::connect(path)
            .await
            .expect("Database connection failed")
    }
    #[cfg(test)]
    pub async fn init_table(&self) {
        self.migrate()
            .await
            .expect("Database migration failed; inspect the existing data before retrying");
    }
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='reservations'",
        )
        .fetch_one(&*self.pool_ref)
        .await?;
        if exists > 0 {
            let bad: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reservations r WHERE r.start>=r.end OR r.start < -62135596800 OR r.end > 253402300799 OR NOT EXISTS(SELECT 1 FROM users WHERE id=r.user_id) OR NOT EXISTS(SELECT 1 FROM resources WHERE id=r.resource_id)")
                .fetch_one(&*self.pool_ref).await?;
            if bad > 0 {
                return Err(sqlx::Error::Protocol(format!("Existing DB has {bad} invalid reservations. Back up and repair before migration.")));
            }
            let overlapping: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reservations a JOIN reservations b ON a.resource_id=b.resource_id AND a.id<b.id AND a.start<b.end AND a.end>b.start")
                .fetch_one(&*self.pool_ref).await?;
            if overlapping > 0 {
                return Err(sqlx::Error::Protocol(format!(
                    "Existing DB has {overlapping} overlapping reservation pairs; inspect the backup before migration"
                )));
            }
        }
        sqlx::migrate!("./migrations")
            .run(&*self.pool_ref)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        Ok(())
    }
    pub async fn settings(&self) -> Result<crate::model::Settings, sqlx::Error> {
        sqlx::query_as("SELECT lab_name,advance_days,slot_minutes,contact_text,notification_text FROM app_settings JOIN reservation_settings USING(id) WHERE id=1")
            .fetch_one(&*self.pool_ref).await
    }
}
