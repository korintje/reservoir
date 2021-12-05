use actix_web::{HttpResponse, Result};
use sqlx::postgres::PgPoolOptions;
use std::fs;
use crate::models::*;

pub fn get_psql_url() -> String {
  fs::read_to_string("sqlinfo.config")
      .expect("Failed to load setting")
}

pub fn internal_server_error(code: i32, message: &str) -> HttpResponse {
let error_json = MyError{code, message: message.to_string()};
HttpResponse::InternalServerError().json(error_json)
}

pub async fn get_pool(url: &str) -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
PgPoolOptions::new()
    .max_connections(5)
    .connect(url)
    .await
}