use crate::models::{User};
use crate::db_handler::{DataAccessor};

impl DataAccessor {

  pub async fn get_user(&self, user_id: i32) -> Result<User, sqlx::Error> {
    sqlx::query_as("SELECT id, user_name FROM users WHERE id=$1")
      .bind(user_id)
      .fetch_one(&*self.pool_ref)
      .await
  }

  pub async fn get_users(&self) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as("SELECT id, user_name FROM users")
      .fetch_all(&*self.pool_ref)
      .await
  }

  pub async fn add_user(&self, user: User) -> Result<User, sqlx::Error> {
    sqlx::query_as("INSERT INTO users (user_name) VALUES ($1) 
                    RETURNING id, user_name"
                  )
      .bind(user.user_name)
      .fetch_one(&*self.pool_ref)
      .await
  }

  pub async fn delete_user(&self, user_id: i32) -> Result<User, sqlx::Error> {
    sqlx::query_as("DELETE FROM users WHERE id=$1 RETURNING id, user_name")
      .bind(user_id)
      .fetch_one(&*self.pool_ref)
      .await
  }

}