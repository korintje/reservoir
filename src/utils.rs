use actix_web::{HttpResponse, Result};
use sqlx::sqlite::SqlitePoolOptions;
use std::fs;
use crate::models::*;
use sha2::{Sha256, Digest};

pub fn get_db_url() -> String {
  "./db/reservoir.db".to_string()
  // fs::read_to_string("sqlinfo.config")
  //    .expect("Failed to load setting")
}

pub fn internal_server_error(code: i32, message: &str) -> HttpResponse {
  let error_json = MyError{code, message: message.to_string()};
  HttpResponse::InternalServerError().json(error_json)
}

pub async fn get_pool(url: &str) -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
  SqlitePoolOptions::new()
    .max_connections(5)
    .connect(url)
    .await
}

pub fn hash(input: &Option<String>) -> String {
  let mut hashed = Sha256::new();
  let string = match input {
    Some(s) => s,
    None => "",
  };
  hashed.update(string);
  format!("{:X}", hashed.finalize())
}