use crate::models::{Reservation, Period, PassHash};
use crate::db_handler::{DataAccessor};

/*
"SELECT reservations.id AS id, user_name AS title, start_datetime AS start, end_datetime AS end, description 
FROM reservations JOIN users ON reservations.user_id=users.id
WHERE (reservation_id=$1) AND ((start_datetime BETWEEN $2 AND $3) OR (end_datetime BETWEEN $2 AND $3))"
*/

impl DataAccessor {

  pub async fn get_reservation_by_id(&self, reservation_id: i32) -> Result<Reservation, sqlx::Error> {
    sqlx::query_as(
      "SELECT reservations.id AS id, user_id, user_name, resource_id, resource_name, start_datetime, end_datetime, description
      FROM reservations JOIN users ON (reservations.user_id=users.id) JOIN resources ON (reservations.resource_id=resource.id) 
      WHERE (reservations.id=$1)"
    )
    .bind(reservation_id)
    .fetch_one(&*self.pool_ref)
    .await
  }

  pub async fn get_reservations(&self) -> Result<Vec<Reservation>, sqlx::Error> {
    sqlx::query_as(
      "SELECT reservations.id AS id, user_id, user_name, resource_id, resource_name, start_datetime, end_datetime, description
      FROM reservations JOIN users ON (reservations.user_id=users.id) JOIN resources ON (reservations.resource_id=resource.id)"
    )
    .fetch_all(&*self.pool_ref)
    .await
  }

  pub async fn get_reservations_by_period(&self, resource_id: i32, period: Period) -> Result<Vec<Reservation>, sqlx::Error> {
    sqlx::query_as(
      "SELECT reservations.id AS id, user_id, user_name, resource_id, resource_name, start_datetime, end_datetime, description
      FROM reservations JOIN users ON reservations.user_id=users.id JOIN resources ON (reservations.resource_id=resource.id) 
      WHERE (resource_id=$1) AND ((start_datetime BETWEEN $2 AND $3) OR (end_datetime BETWEEN $2 AND $3))"
    )
    .bind(resource_id)
    .bind(&period.from)
    .bind(&period.until)
    .fetch_all(&*self.pool_ref)
    .await
  }

  pub async fn add_reservation(&self, reservation: Reservation, passhash: String) -> Result<Reservation, sqlx::Error> {
    println!("Here!");
    sqlx::query_as(
      "INSERT INTO reservations (resource_id, user_id, start_datetime, end_datetime, description, passhash) VALUES ($1, $2, $3, $4, $5, $6) 
      RETURNING id, resource_id, user_id, start_datetime, end_datetime, description"
    )
    .bind(reservation.resource_id)
    .bind(reservation.user_id)
    .bind(reservation.start_datetime)
    .bind(reservation.end_datetime)
    .bind(reservation.description)
    .bind(passhash)
    .fetch_one(&*self.pool_ref)
    .await
  }

  pub async fn get_passhash_by_id(&self, reservation_id: i32) -> Result<PassHash, sqlx::Error> {
    sqlx::query_as(
      "SELECT passhash FROM reservations WHERE (reservations.id=$1)"
    )
    .bind(reservation_id)
    .fetch_one(&*self.pool_ref)
    .await
  }

  pub async fn delete_reservation(&self, reservation_id: i32) -> Result<Reservation, sqlx::Error> {
    sqlx::query_as(
      "DELETE FROM reservations WHERE id=$1 
      RETURNING id, resource_id, resource_name, user_id, user_name, start_datetime, end_datetime, description"
    )
    .bind(reservation_id)
    .fetch_one(&*self.pool_ref)
    .await
  }

}
  /*
  pub async fn delete_reservation(&self, reservation_id: i32, password: Option<String>) -> Result<Reservation, sqlx::Error> {
    let mut challenge_passhash = Sha256::new();
    if let Some(password) = password {
        challenge_passhash.update(&password);
    }
    let stored_passhash = sqlx::query_as("SELECT passhash FROM reservations WHERE $1=id")
                            .bind(reservation_id)
                            .fetch_one(&*self.pool_ref)
                            .await;
    if let Ok(stored_passhash) = stored_passhash {
        if format!("{:X}", challenge_passhash.finalize()) == stored_passhash {
            sqlx::query_as("DELETE FROM reservations WHERE id=$1
                            RETURNING id, resource_id, resource_name, user_id, user_name, start_datetime, end_datetime, description"
                          )
                          .bind(reservation_id)
                          .fetch_one(&*self.pool_ref)
                          .await
        }
    }
  */

/*
"SELECT id, passhash FROM reservations WHERE $1=id"

"DELETE FROM reservations WHERE $1=id"




}

#[get("/fullcalendar_events")]
async fn get_fullcalendar_events(period: web::Query<FullCalendarPeriod>) -> HttpResponse {
    /*Specialized getter method for JS Fullcalendar library*/
    let psql_url: &str = &utils::get_psql_url();
    let pool  = match utils::get_pool(psql_url).await {
        Ok(p) => p,
        Err(e) => return utils::internal_server_error(4001, &e.to_string()),
    };
    let events: Vec<FullCalendarEvent> = match sqlx::query_as(
)
        .bind(&period.reservation)
        .bind(&period.start)
        .bind(&period.end)
        .fetch_all(&pool).await {
            Ok(item) => item,
            Err(e) => return utils::internal_server_error(4002, &e.to_string()),
        };
    HttpResponse::Ok().json(events)
}

#[post("/reservation")]
async fn post_reservation(info: web::Query<ReservationInfo>) -> HttpResponse {
    println!("here");
    let mut passhash = Sha256::new();
    passhash.update(&info.password);
    let psql_url: &str = &utils::get_psql_url();
    let pool  = match utils::get_pool(psql_url).await {
        Ok(p) => p,
        Err(e) => return utils::internal_server_error(4001, &e.to_string()),
    };
    let output: ReservationOut = match sqlx::query_as(
        "INSERT INTO reservations (reservation_id, user_id, start_datetime, end_datetime, description, passhash)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id")
        .bind(&info.reservation_id)
        .bind(&info.user_id)
        .bind(&info.start_datetime)
        .bind(&info.end_datetime)
        .bind(&info.description)
        .bind(format!("{:X}", passhash.finalize()))
        .fetch_one(&pool).await {
            Ok(out) => out,
            Err(e) => return utils::internal_server_error(4002, &e.to_string()),
        };
    HttpResponse::Ok().body(format!("reserved! ID: {}\n", output.id))
}

#[delete("/reservation")]
async fn delete_reservation(info: web::Query<DeleteInfo>) -> HttpResponse {
    let mut posted_passhash = Sha256::new();
    posted_passhash.update(&info.password);
    let psql_url: &str = &utils::get_psql_url();
    let pool  = match utils::get_pool(psql_url).await {
        Ok(p) => p,
        Err(e) => return utils::internal_server_error(4001, &e.to_string()),
    };
    let stored: StoredPass = match sqlx::query_as(
        "SELECT id, passhash FROM reservations WHERE $1=id")
        .bind(&info.id).fetch_one(&pool).await{
            Ok(item) => item,
            Err(e) => return utils::internal_server_error(4002, &e.to_string())
        };
    if (format!("{:X}", posted_passhash.finalize()) == stored.passhash) {
        let output: ReservationOut = match sqlx::query_as(
            "DELETE FROM reservations WHERE $1=id")
            .bind(&info.id).fetch_one(&pool).await{
                Ok(o) => o,
                Err(e) => return utils::internal_server_error(4002, &e.to_string())
            };   
        return HttpResponse::Ok().body(format!("reserved! ID: {}\n", output.id))     
    }else{
        return HttpResponse::Unauthorized().finish()
    }    
}
*/
