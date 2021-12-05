use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use crate::models::{Resource, ID};

pub struct DataAccessor {
  pub pool_ref: Arc<sqlx::Pool<sqlx::Postgres>>,
}

impl DataAccessor {
  
  pub async fn new(psql_url: &str) -> DataAccessor {
    let pool = PgPoolOptions::new().max_connections(5).connect(psql_url).await.unwrap();
    let pool_ref = Arc::new(pool);
    DataAccessor {
      pool_ref: pool_ref,
    }
  }

  pub async fn get_resource_by_id(&self, resource_id: &i32) -> Result<Resource, sqlx::Error> {
    sqlx::query_as("SELECT id, resource_name FROM resources WHERE id=$1")
      .bind(resource_id)
      .fetch_one(&*self.pool_ref)
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

}