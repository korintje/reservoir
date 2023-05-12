use crate::model::{User};
use crate::db_handler::{DataAccessor};

impl DataAccessor {

  pub async fn get_user(&self, user_id: i32) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT id, user_name, active FROM users WHERE id=$1")
      .bind(user_id)
      .fetch_one(&*self.pool_ref)
      .await
  }

  pub async fn get_users(&self) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as("SELECT id, user_name, active FROM users")
      .fetch_all(&*self.pool_ref)
      .await
  }

  pub async fn add_user(&self, user: User) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO users (user_name, active) VALUES ($1, $2)")
      .bind(user.user_name)
      .bind(user.active)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn delete_user(&self, user_id: i32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id=$1")
      .bind(user_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_user_name(&self, user_id: i32, new_name: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE users SET user_name=$1 WHERE id=$2")
      .bind(new_name)
      .bind(user_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_user_activity(&self, user_id: i32, activity: bool) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE users SET active=$1 WHERE id=$2")
      .bind(activity)
      .bind(user_id)
      .execute(&*self.pool_ref)
      .await
  }

}