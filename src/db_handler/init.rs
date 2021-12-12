use crate::model::{TableCount};
use crate::db_handler::{DataAccessor};

impl DataAccessor {

  pub async fn init_table(&self) {
    if !self.is_table_exist("reservations").await {
      self.create_reservations_table().await.unwrap();
    }
    if !self.is_table_exist("resources").await {
      self.create_resources_table().await.unwrap();
    }
    if !self.is_table_exist("users").await {
      self.create_users_table().await.unwrap();
    }
  }

  pub async fn is_table_exist(&self, table_name: &str) -> bool {
    let table_count: TableCount = sqlx::query_as(
      "SELECT COUNT(*) as count FROM sqlite_master WHERE TYPE='table' AND name=$1"
    )
    .bind(table_name)
    .fetch_one(&*self.pool_ref)
    .await.unwrap();
    if table_count.count == 0 {false} else {true}
  }
  
  pub async fn create_reservations_table(&self) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query(
      "CREATE TABLE reservations (
        id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
        , user_id       INTEGER NOT NULL
        , resource_id   INTEGER NOT NULL
        , start         INTEGER NOT NULL
        , end           INTEGER NOT NULL
        , description   TEXT
        , passhash      TEXT
      )"
    )
    .execute(&*self.pool_ref)
    .await
  }
  
  pub async fn create_resources_table(&self) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query(
      "CREATE TABLE resources (
        id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
        , resource_name TEXT NOT NULL
        , active        INTEGER NOT NULL
      )"
    )
    .execute(&*self.pool_ref)
    .await
  }
  
  pub async fn create_users_table(&self) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query(
      "CREATE TABLE users (
        id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
        , user_name     TEXT NOT NULL
        , active        INTEGER NOT NULL
      )"
    )
    .execute(&*self.pool_ref)
    .await
  }
  
}

