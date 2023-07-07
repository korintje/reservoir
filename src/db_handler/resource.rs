use crate::model::{Resource};
use crate::db_handler::{DataAccessor};

impl DataAccessor {

  pub async fn get_resource(&self, resource_id: i32) -> Result<Resource, sqlx::Error> {
    sqlx::query_as("SELECT id, resource_name, active, capacity, custom_field FROM resources WHERE id=$1")
      .bind(resource_id)
      .fetch_one(&*self.pool_ref)
      .await
  }

  pub async fn get_resources(&self) -> Result<Vec<Resource>, sqlx::Error> {
    sqlx::query_as("SELECT id, resource_name, active, capacity, custom_field FROM resources")
      .fetch_all(&*self.pool_ref)
      .await
  }

  pub async fn add_resource(&self, resource: Resource) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO resources (resource_name, active, capacity, custom_field) VALUES ($1, $2, $3, $4)")
      .bind(resource.resource_name)
      .bind(resource.active)
      .bind(resource.capacity)
      .bind(resource.custom_field)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn delete_resource(&self, resource_id: i32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("DELETE FROM resources WHERE id=$1")
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_resource_name(&self, resource_id: i32, new_name: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE resources SET resource_name=$1 WHERE id=$2")
      .bind(new_name)
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_resource_activity(&self, resource_id: i32, activity: bool) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE resources SET active=$1 WHERE id=$2")
      .bind(activity)
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_resource_capacity(&self, resource_id: i32, capacity: i32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE resources SET capacity=$1 WHERE id=$2")
      .bind(capacity)
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_resource_custom_field(&self, resource_id: i32, custom_field: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE resources SET custom_field=$1 WHERE id=$2")
      .bind(custom_field)
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

}