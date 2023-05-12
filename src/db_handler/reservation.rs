use crate::model::{ReservationPost, ReservationReturn, ReservationDB, Filter, PassHash};
use crate::db_handler::{DataAccessor};

impl DataAccessor {

  pub async fn get_reservation(&self, reservation_id: i32) -> Result<ReservationReturn, sqlx::Error> {
    sqlx::query_as(
      "SELECT reservations.id AS id, user_id, user_name, resource_id, resource_name, start, end, description, passhash
      FROM reservations JOIN users ON (reservations.user_id=users.id) JOIN resources ON (reservations.resource_id=resources.id) 
      WHERE (reservations.id=$1)"
    )
    .bind(reservation_id)
    .fetch_one(&*self.pool_ref)
    .await
    .map(|obj: ReservationDB| ReservationReturn::from(obj))
  }

  pub async fn get_reservations(&self, filter: Filter) -> Result<Vec<ReservationReturn>, sqlx::Error> {
    let mut query = 
      "SELECT reservations.id AS id, user_id, user_name, resource_id, resource_name, start, end, description, passhash 
      FROM reservations JOIN users ON (reservations.user_id=users.id) JOIN resources ON (reservations.resource_id=resources.id)"
      .to_string();
    let mut options = Vec::new();
    if let Some(dt) = filter.from { options.push(format!("end>{}", dt.timestamp())); }
    if let Some(dt) = filter.until { options.push(format!("start<={}", dt.timestamp())); }
    if let Some(id) = filter.user_id { options.push(format!("user_id={}", id)); }
    if let Some(id) = filter.resource_id { options.push(format!("resource_id={}", id)); }
    if !options.is_empty() {
      query += " WHERE ";
      query += &options.join(" AND ");
    }
    sqlx::query_as(&query)
    .fetch_all(&*self.pool_ref)
    .await
    .map(|v| {
      v.into_iter().map(|obj: ReservationDB| {
        ReservationReturn::from(obj)
      }).collect()
    })
  }

  pub async fn add_reservation(&self, reservation: ReservationPost) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    let reservation_db = ReservationDB::from(reservation);
    sqlx::query(
      "INSERT INTO reservations (resource_id, user_id, start, end, description, passhash) 
      VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(reservation_db.resource_id)
    .bind(reservation_db.user_id)
    .bind(reservation_db.start)
    .bind(reservation_db.end)
    .bind(reservation_db.description)
    .bind(reservation_db.passhash)
    .execute(&*self.pool_ref)
    .await
  }

  pub async fn get_passhash_by_id(&self, reservation_id: i32) -> Result<PassHash, sqlx::Error> {
    sqlx::query_as(
      "SELECT passhash FROM reservations WHERE id=$1"
    )
    .bind(reservation_id)
    .fetch_one(&*self.pool_ref)
    .await
  }

  pub async fn delete_reservation(&self, reservation_id: i32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query(
      "DELETE FROM reservations WHERE id=$1"
    )
    .bind(reservation_id)
    .execute(&*self.pool_ref)
    .await
  }

  pub async fn update_user(&self, reservation_id: i32, new_user: i32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE reservations SET user_id=$1 WHERE id=$2")
      .bind(new_user)
      .bind(reservation_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_resource(&self, reservation_id: i32, new_resource: i32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE reservations SET resource_id=$1 WHERE id=$2")
      .bind(new_resource)
      .bind(reservation_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_start(&self, reservation_id: i32, new_start: i64) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE reservations SET start=$1 WHERE id=$2")
      .bind(new_start)
      .bind(reservation_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_end(&self, reservation_id: i32, new_end: i64) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE reservations SET end=$1 WHERE id=$2")
      .bind(new_end)
      .bind(reservation_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_description(&self, reservation_id: i32, new_description: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE reservations SET description=$1 WHERE id=$2")
      .bind(new_description)
      .bind(reservation_id)
      .execute(&*self.pool_ref)
      .await
  }

  pub async fn update_passhash(&self, reservation_id: i32, new_passhash: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    sqlx::query("UPDATE reservations SET passhash=$1 WHERE id=$2")
      .bind(new_passhash)
      .bind(reservation_id)
      .execute(&*self.pool_ref)
      .await
  }

}

