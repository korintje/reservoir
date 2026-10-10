use crate::{db_handler::DataAccessor, model::Resource};
impl DataAccessor {
    pub async fn get_resource(&self, id: i32) -> Result<Resource, sqlx::Error> {
        sqlx::query_as("SELECT * FROM resources WHERE id=?")
            .bind(id)
            .fetch_one(&*self.pool_ref)
            .await
    }
    pub async fn get_resources(&self) -> Result<Vec<Resource>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM resources ORDER BY sort_order,resource_name,id")
            .fetch_all(&*self.pool_ref)
            .await
    }
    pub async fn delete_resource(
        &self,
        id: i32,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query("DELETE FROM resources WHERE id=?")
            .bind(id)
            .execute(&*self.pool_ref)
            .await
    }
}
