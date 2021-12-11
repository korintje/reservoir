use crate::models::{Resource};
use crate::db_handler::{DataAccessor};

impl DataAccessor {

  pub async fn get_resource(&self, resource_id: i32) -> Result<Resource, sqlx::Error> {
    sqlx::query_as("SELECT id, resource_name FROM resources WHERE id=$1")
      .bind(resource_id)
      .fetch_one(&*self.pool_ref)
      .await
  }

  pub async fn get_resources(&self) -> Result<Vec<Resource>, sqlx::Error> {
    sqlx::query_as("SELECT id, resource_name FROM resources")
      .fetch_all(&*self.pool_ref)
      .await
  }

  pub async fn add_resource(&self, resource: Resource) -> Result<Resource, sqlx::Error> {
    sqlx::query_as("INSERT INTO resources (resource_name) VALUES ($1) 
                    RETURNING id, resource_name"
                  )
      .bind(resource.resource_name)
      .fetch_one(&*self.pool_ref)
      .await
  }

  pub async fn delete_resource(&self, user_id: i32) -> Result<Resource, sqlx::Error> {
    sqlx::query_as("DELETE FROM resources WHERE id=$1 RETURNING id, resource_name")
      .bind(user_id)
      .fetch_one(&*self.pool_ref)
      .await
  }

}