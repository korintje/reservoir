// use std::process::Command;
use std::sync::Arc;
use sqlx::sqlite::SqlitePoolOptions;

pub mod init;
pub mod resource;
pub mod user;
pub mod reservation;

pub use init::*;
pub use resource::*;
pub use user::*;
pub use reservation::*;

pub struct DataAccessor {
  pub pool_ref: Arc<sqlx::Pool<sqlx::Sqlite>>,
}

impl DataAccessor { 
  
  pub async fn new(db_path: &str) -> DataAccessor {
    let pool = SqlitePoolOptions::new().max_connections(5).connect(db_path).await.unwrap();
    let pool_ref = Arc::new(pool);
    DataAccessor {pool_ref}
  }

}