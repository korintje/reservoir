use chrono::{DateTime, Local};
use actix_web::{get, post, delete, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use sqlx::postgres::PgPoolOptions;
use serde::{Deserialize, Serialize};
use actix_files::NamedFile;
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use std::fs;

mod structs;
use structs::*;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

fn internal_server_error(code: i32, message: &str) -> HttpResponse {
    let error_json = MyError{code, message: message.to_string()};
    HttpResponse::InternalServerError().json(error_json)
}

//fn get_pool() -> std::result::Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error>


#[get("/fullcalendar_events")]
async fn get_fullcalendar_events(period: web::Query<FullCalendarPeriod>) -> HttpResponse {
    /*Specialized getter method for JS Fullcalendar library*/
    let psql_url: &str = &fs::read_to_string("sqlinfo.config").expect("Failed to load setting");
    let pool  = match PgPoolOptions::new().max_connections(5).connect(psql_url).await {
        Ok(p) => p,
        Err(e) => return internal_server_error(4001, &e.to_string()),
    };
    let events: Vec<FullCalendarEvent> = match sqlx::query_as(
        "SELECT reservations.id AS id, member_name AS title, start_datetime AS start, end_datetime AS end, description 
         FROM reservations JOIN members ON reservations.member_id=members.id
         WHERE (machine_id=$1) AND ((start_datetime BETWEEN $2 AND $3) OR (end_datetime BETWEEN $2 AND $3))")
        .bind(&period.machine).bind(&period.start).bind(&period.end)
        .fetch_all(&pool).await {
            Ok(item) => item,
            Err(e) => return internal_server_error(4002, &e.to_string()),
        };
    HttpResponse::Ok().json(events)
}

#[post("/reservation")]
async fn post_reservation(info: web::Query<ReservationInfo>) -> HttpResponse {
    let psql_url: &str = &fs::read_to_string("sqlinfo.config").expect("Failed to load setting");
    let mut passhash = Sha256::new();
    passhash.update(&info.password);
    let pool  = match PgPoolOptions::new()
        .max_connections(5).connect(psql_url).await {
            Ok(p) => p,
            Err(e) => return internal_server_error(4001, &e.to_string()),
        };
    let output: ReservationOut = match sqlx::query_as(
        "INSERT INTO reservations (machine_id, member_id, start_datetime, end_datetime, description, passhash)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id")
        .bind(&info.machine_id).bind(&info.member_id).bind(&info.start_datetime)
        .bind(&info.end_datetime).bind(&info.description).bind(format!("{:X}", passhash.finalize()))
        .fetch_one(&pool).await {
            Ok(out) => out,
            Err(e) => return internal_server_error(4002, &e.to_string()),
        };
    HttpResponse::Ok().body(format!("reserved! ID: {}\n", output.id))
}

#[delete("/reservation")]
async fn delete_reservation(info: web::Query<DeleteInfo>) -> HttpResponse {
    let psql_url: &str = &fs::read_to_string("sqlinfo.config").expect("Failed to load setting");
    let mut posted_passhash = Sha256::new();
    posted_passhash.update(&info.password);
    let pool = match PgPoolOptions::new()
        .max_connections(5).connect(psql_url).await{
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        App::new()
            .service(get_fullcalendar_events)
            .service(post_reservation)
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}