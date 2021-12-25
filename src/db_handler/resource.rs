use crate::model::{Resource};
use crate::db_handler::{DataAccessor};

impl DataAccessor {

  pub async fn get_resource(&self, resource_id: i32) -> Result<Resource, sqlx::Error> {
    sqlx::query_as("SELECT id, resource_name, active FROM resources WHERE id=$1")
      .bind(resource_id)
      .fetch_one(&*self.pool_ref)
      .await
  }

  pub async fn get_resources(&self) -> Result<Vec<Resource>, sqlx::Error> {
    sqlx::query_as("SELECT id, resource_name, active FROM resources")
      .fetch_all(&*self.pool_ref)
      .await
  }

  pub async fn add_resource(&self, resource: Resource) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query("INSERT INTO resources (resource_name, active) VALUES ($1, $2)")
      .bind(resource.resource_name)
      .bind(resource.active)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn delete_resource(&self, resource_id: i32) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query("DELETE FROM resources WHERE id=$1")
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_resource_name(&self, resource_id: i32, new_name: &str) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query("UPDATE resources SET resource_name=$1 WHERE id=$2")
      .bind(new_name)
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_resource_activity(&self, resource_id: i32, activity: bool) -> Result<sqlx::sqlite::SqliteDone, sqlx::Error> {
    sqlx::query("UPDATE resources SET active=$1 WHERE id=$2")
      .bind(activity)
      .bind(resource_id)
      .execute(&*self.pool_ref)
      .await
  }

}