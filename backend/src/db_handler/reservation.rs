use crate::{db_handler::DataAccessor, model::*};
use chrono::Utc;
impl DataAccessor {
    pub async fn get_reservation(&self, id: i32) -> Result<ReservationReturn, sqlx::Error> {
        let row:ReservationDB=sqlx::query_as("SELECT r.id,r.user_id,u.user_name,r.resource_id,s.resource_name,r.start,r.end,r.description,r.created_at,r.version FROM reservations r JOIN users u ON u.id=r.user_id JOIN resources s ON s.id=r.resource_id WHERE r.id=?")
            .bind(id).fetch_one(&*self.pool_ref).await?;
        row.try_into()
    }
    pub async fn get_reservations(&self, f: Filter) -> Result<Vec<ReservationReturn>, sqlx::Error> {
        let rows:Vec<ReservationDB>=sqlx::query_as("SELECT r.id,r.user_id,u.user_name,r.resource_id,s.resource_name,r.start,r.end,r.description,r.created_at,r.version FROM reservations r JOIN users u ON u.id=r.user_id JOIN resources s ON s.id=r.resource_id WHERE (? IS NULL OR r.end>?) AND (? IS NULL OR r.start<?) AND (? IS NULL OR r.user_id=?) AND (? IS NULL OR r.resource_id=?) ORDER BY r.start,r.id LIMIT 5000")
            .bind(f.from.map(|v|v.timestamp())).bind(f.from.map(|v|v.timestamp()))
            .bind(f.until.map(|v|v.timestamp())).bind(f.until.map(|v|v.timestamp()))
            .bind(f.user_id).bind(f.user_id).bind(f.resource_id).bind(f.resource_id)
            .fetch_all(&*self.pool_ref).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
    pub async fn add_reservation_hashed(
        &self,
        r: ReservationPost,
        passhash: String,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query("INSERT INTO reservations(resource_id,user_id,start,end,description,passhash,created_at) VALUES(?,?,?,?,?,?,?)")
            .bind(r.resource_id).bind(r.user_id).bind(r.start.timestamp()).bind(r.end.timestamp())
            .bind(r.description).bind(passhash).bind(Utc::now().timestamp()).execute(&*self.pool_ref).await
    }
    #[cfg(test)]
    pub async fn add_reservation(
        &self,
        r: ReservationPost,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let hash = crate::utils::hash_anyway(&r.password);
        self.add_reservation_hashed(r, hash).await
    }
    pub async fn get_passhash_by_id(&self, id: i32) -> Result<PassHash, sqlx::Error> {
        sqlx::query_as("SELECT passhash,version FROM reservations WHERE id=?")
            .bind(id)
            .fetch_one(&*self.pool_ref)
            .await
    }
    #[cfg(test)]
    pub async fn delete_reservation(
        &self,
        id: i32,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query("DELETE FROM reservations WHERE id=?")
            .bind(id)
            .execute(&*self.pool_ref)
            .await
    }
    #[cfg(test)]
    pub async fn update_reservation_atomic(
        &self,
        id: i32,
        r: ReservationPut,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query("UPDATE reservations SET user_id=COALESCE(?,user_id),resource_id=COALESCE(?,resource_id),start=COALESCE(?,start),end=COALESCE(?,end),description=COALESCE(?,description) WHERE id=?")
            .bind(r.user_id).bind(r.resource_id).bind(r.start.map(|v|v.timestamp())).bind(r.end.map(|v|v.timestamp())).bind(r.description).bind(id).execute(&*self.pool_ref).await
    }
}
