use actix_web::{get, post, delete, web, HttpResponse, Responder};
use sha2::{Sha256, Digest};
use crate::models::*;
use crate::objects::{DataAccessor};

#[get("/resource/{id}")]
async fn get_resource(resource_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_resource_by_id(&resource_id).await;
    match result {
        Err(e) => HttpResponse::NotFound().json(MyError{code: 4004, message: e.to_string()}),
        Ok(resource) => HttpResponse::Ok().json(resource),
    }
}

#[post("/resource")]
async fn add_resource(resource: web::Query<Resource>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.add_resource(resource.into_inner()).await;
    match result {
        Err(e) => HttpResponse::InternalServerError().json(MyError{code: 4000, message: e.to_string()}),
        Ok(resource) => HttpResponse::Ok().json(resource),
    }
}