use actix_web::{get, post, delete, web, HttpResponse, Result};
use sqlx::postgres::PgPoolOptions;
use sha2::{Sha256, Digest};
use std::fs;
use crate::models::*;

fn get_psql_url() -> String {
    fs::read_to_string("sqlinfo.config")
        .expect("Failed to load setting")
}

fn internal_server_error(code: i32, message: &str) -> HttpResponse {
  let error_json = MyError{code, message: message.to_string()};
  HttpResponse::InternalServerError().json(error_json)
}

async fn get_pool(url: &str) -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
  PgPoolOptions::new()
      .max_connections(5)
      .connect(url)
      .await
}

#[get("/fullcalendar_events")]
async fn get_fullcalendar_events(period: web::Query<FullCalendarPeriod>) -> HttpResponse {
    /*Specialized getter method for JS Fullcalendar library*/
    let psql_url: &str = &get_psql_url();
    let pool  = match get_pool(psql_url).await {
        Ok(p) => p,
        Err(e) => return internal_server_error(4001, &e.to_string()),
    };
    let events: Vec<FullCalendarEvent> = match sqlx::query_as(
        "SELECT reservations.id AS id, user_name AS title, start_datetime AS start, end_datetime AS end, description 
         FROM reservations JOIN users ON reservations.user_id=users.id
         WHERE (resource_id=$1) AND ((start_datetime BETWEEN $2 AND $3) OR (end_datetime BETWEEN $2 AND $3))")
        .bind(&period.resource)
        .bind(&period.start)
        .bind(&period.end)
        .fetch_all(&pool).await {
            Ok(item) => item,
            Err(e) => return internal_server_error(4002, &e.to_string()),
        };
    HttpResponse::Ok().json(events)
}

#[post("/reservation")]
async fn post_reservation(info: web::Query<ReservationInfo>) -> HttpResponse {
    let mut passhash = Sha256::new();
    passhash.update(&info.password);
    let psql_url: &str = &get_psql_url();
    let pool  = match get_pool(psql_url).await {
        Ok(p) => p,
        Err(e) => return internal_server_error(4001, &e.to_string()),
    };
    let output: ReservationOut = match sqlx::query_as(
        "INSERT INTO reservations (resource_id, user_id, start_datetime, end_datetime, description, passhash)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id")
        .bind(&info.resource_id)
        .bind(&info.user_id)
        .bind(&info.start_datetime)
        .bind(&info.end_datetime)
        .bind(&info.description)
        .bind(format!("{:X}", passhash.finalize()))
        .fetch_one(&pool).await {
            Ok(out) => out,
            Err(e) => return internal_server_error(4002, &e.to_string()),
        };
    HttpResponse::Ok().body(format!("reserved! ID: {}\n", output.id))
}

#[delete("/reservation")]
async fn delete_reservation(info: web::Query<DeleteInfo>) -> HttpResponse {
    let mut posted_passhash = Sha256::new();
    posted_passhash.update(&info.password);
    let psql_url: &str = &get_psql_url();
    let pool  = match get_pool(psql_url).await {
        Ok(p) => p,
        Err(e) => return internal_server_error(4001, &e.to_string()),
    };
    let stored: StoredPass = match sqlx::query_as(
        "SELECT id, passhash FROM reservations WHERE $1=id")
        .bind(&info.id).fetch_one(&pool).await{
            Ok(item) => item,
            Err(e) => return internal_server_error(4002, &e.to_string())
        };
    if (format!("{:X}", posted_passhash.finalize()) == stored.passhash) {
        let output: ReservationOut = match sqlx::query_as(
            "DELETE FROM reservations WHERE $1=id")
            .bind(&info.id).fetch_one(&pool).await{
                Ok(o) => o,
                Err(e) => return internal_server_error(4002, &e.to_string())
            };   
        return HttpResponse::Ok().body(format!("reserved! ID: {}\n", output.id))     
    }else{
        return HttpResponse::Unauthorized().finish()
    }    
}