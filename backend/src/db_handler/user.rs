use crate::{db_handler::DataAccessor, model::User};
impl DataAccessor {
    pub async fn get_user(&self, id: i32) -> Result<User, sqlx::Error> {
        sqlx::query_as("SELECT * FROM users WHERE id=?")
            .bind(id)
            .fetch_one(&*self.pool_ref)
            .await
    }
    pub async fn get_users(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM users ORDER BY sort_order,user_name,id")
            .fetch_all(&*self.pool_ref)
            .await
    }
    pub async fn delete_user(
        &self,
        id: i32,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id=?")
            .bind(id)
            .execute(&*self.pool_ref)
            .await
    }
}
