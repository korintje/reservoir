use actix_web::{get, post, delete, put, web, HttpResponse, Responder};
use crate::model::{Resource};
use crate::db_handler::{DataAccessor};
use crate::{response};
use response::{MyResponse};

#[get("/resources/{id}")]
async fn get_resource(resource_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_resource(resource_id.into_inner()).await;
    match result {
        Err(_) => MyResponse::item_not_found(),
        Ok(resource) => HttpResponse::Ok().json(resource),
    }
}

#[get("/resources")]
async fn get_resources(accessor: web::Data<DataAccessor>) -> impl Responder {
    let result = accessor.get_resources().await;
    match result {
        Err(_) => MyResponse::item_not_found(),
        Ok(resource) => HttpResponse::Ok().json(resource),
    }
}

#[post("/resources")]
async fn add_resource(resource: web::Json<Resource>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let resource = resource.into_inner();
    let result = accessor.add_resource(resource).await;
    match result {
        Err(e) => MyResponse::bad_request(&e.to_string()),
        Ok(_) => MyResponse::ok(),
    }
}

#[delete("/resources/{id}")]
async fn delete_resource(resource_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
    let resource_id = resource_id.into_inner();
    let result = accessor.delete_resource(resource_id).await;
    match result {
        Err(_) => MyResponse::item_not_found(),
        Ok(_) => MyResponse::ok(),
    }
}

#[put("/resources/{id}")]
async fn update_resource(resource_id: web::Path<i32>, resource: web::Json<Resource>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let resource_id = resource_id.into_inner();
  let resource = resource.into_inner();
  if let Some(name) = resource.resource_name {
    let result = accessor.update_resource_name(resource_id, &name).await;
    if let Err(e) = result {
      return MyResponse::bad_request(&e.to_string());
    }
  }
  if let Some(activity) = resource.active {
    let result = accessor.update_resource_activity(resource_id, activity).await;
    if let Err(e) = result {
      return MyResponse::bad_request(&e.to_string());
    }
  }
  MyResponse::ok()
}

