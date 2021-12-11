use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::models::{Resource, MyError, MySuccess};
use crate::db_handler::{DataAccessor};

#[get("/resources/{id}")]
async fn get_resource(resource_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_resource(resource_id.into_inner()).await;
    match result {
        Err(e) => HttpResponse::NotFound().json(MyError{code: 4004, message: e.to_string()}),
        Ok(resource) => HttpResponse::Ok().json(resource),
    }
}

#[get("/resources")]
async fn get_resources(accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_resources().await;
    match result {
        Err(e) => HttpResponse::InternalServerError().json(MyError{code: 4000, message: e.to_string()}),
        Ok(resource) => HttpResponse::Ok().json(resource),
    }
}

#[post("/resources")]
async fn add_resource(resource: web::Json<Resource>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let resource = resource.into_inner();
    let resource_id = resource.id;
    let result = accessor.add_resource(resource).await;
    match result {
        Err(e) => HttpResponse::InternalServerError().json(MyError{code: 4000, message: e.to_string()}),
        Ok(_) => HttpResponse::Ok().json(MySuccess{code: 2000, message: format!("User {} has successfully added", resource_id)}),
    }
}

#[delete("/resources/{id}")]
async fn delete_resource(resource_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let resource_id = resource_id.into_inner();
    let result = accessor.delete_resource(resource_id).await;
    match result {
        Err(e) => HttpResponse::NotFound().json(MyError{code: 4004, message: e.to_string()}),
        Ok(_) => HttpResponse::Ok().json(MySuccess{code: 2000, message: format!("User {} has successfully added", resource_id)}),
    }
}