use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::models::{User, MyError, MySuccess};
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
    println!("USER POSTED");
    let user = user.into_inner();
    let user_id = user.id;
    let result = accessor.add_user(user).await;
    match result {
        Err(e) => HttpResponse::InternalServerError().json(MyError{code: 4000, message: e.to_string()}),
        Ok(_) => HttpResponse::Ok().json(MySuccess{code: 2000, message: format!("User {} has successfully added", user_id)}),
    }
}

#[delete("/users/{id}")]
async fn delete_user(user_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let user_id = user_id.into_inner();
    let result = accessor.delete_user(user_id).await;
    match result {
        Err(e) => HttpResponse::NotFound().json(MyError{code: 4004, message: e.to_string()}),
        Ok(_) => HttpResponse::Ok().json(MySuccess{code: 2000, message: format!("User {} has successfully removed", user_id)}),
    }
}