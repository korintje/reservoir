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

  pub async fn add_user(&self, user: User) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query("INSERT INTO users (user_name, active) VALUES ($1, $2)")
      .bind(user.user_name)
      .bind(user.active)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn delete_user(&self, user_id: i32) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id=$1")
      .bind(user_id)
      .execute(&*self.pool_ref)
      .await
  }

}