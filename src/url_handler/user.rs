use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::model::{User};
use crate::db_handler::{DataAccessor};
use crate::{response};
use response::{MyResponse};

#[get("/users/{id}")]
async fn get_user(user_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_user(user_id.into_inner()).await;
    match result {
        Err(_) => MyResponse::ItemNotFound(),
        Ok(user) => HttpResponse::Ok().json(user),
    }
}

#[get("/users")]
async fn get_users(accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_users().await;
    match result {
        Err(_) => MyResponse::ItemNotFound(),
        Ok(user) => HttpResponse::Ok().json(user),
    }
}

#[post("/users")]
async fn add_user(user: web::Json<User>, accessor: web::Data<DataAccessor>) -> impl Responder {
    println!("USER POSTED");
    let user = user.into_inner();
    let result = accessor.add_user(user).await;
    match result {
        Err(e) => MyResponse::BadRequest(&e.to_string()),
        Ok(_) => MyResponse::Ok(),
    }
}

#[delete("/users/{id}")]
async fn delete_user(user_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let user_id = user_id.into_inner();
    let result = accessor.delete_user(user_id).await;
    match result {
        Err(_) => MyResponse::ItemNotFound(),
        Ok(_) => MyResponse::Ok(),
    }
}