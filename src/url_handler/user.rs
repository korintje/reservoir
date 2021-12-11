use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::models::{User, MyError};
use crate::db_handler::{DataAccessor};

#[get("/users/{id}")]
async fn get_user(user_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_user(user_id.into_inner()).await;
    match result {
        Err(e) => HttpResponse::NotFound().json(MyError{code: 4004, message: e.to_string()}),
        Ok(user) => HttpResponse::Ok().json(user),
    }
}

#[get("/users")]
async fn get_users(accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_users().await;
    match result {
        Err(e) => HttpResponse::InternalServerError().json(MyError{code: 4000, message: e.to_string()}),
        Ok(user) => HttpResponse::Ok().json(user),
    }
}

#[post("/users")]
async fn add_user(user: web::Json<User>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.add_user(user.into_inner()).await;
    match result {
        Err(e) => HttpResponse::InternalServerError().json(MyError{code: 4000, message: e.to_string()}),
        Ok(user) => HttpResponse::Ok().json(user),
    }
}

#[delete("/users/{id}")]
async fn delete_user(user: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.delete_user(user.into_inner()).await;
    match result {
        Err(e) => HttpResponse::NotFound().json(MyError{code: 4004, message: e.to_string()}),
        Ok(user) => HttpResponse::Ok().json(user),
    }
}