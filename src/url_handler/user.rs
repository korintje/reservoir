use actix_web::{get, post, delete, put, web, HttpResponse, Responder};
use crate::model::{User};
use crate::db_handler::{DataAccessor};
use crate::{response};
use response::{MyResponse};

#[get("/users/{id}")]
async fn get_user(user_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let result = accessor.get_user(user_id.into_inner()).await;
  match result {
    Err(_) => MyResponse::item_not_found(),
    Ok(user) => HttpResponse::Ok().json(user),
  }
}

#[get("/users")]
async fn get_users(accessor: web::Data<DataAccessor>) -> impl Responder {
  let result = accessor.get_users().await;
  match result {
    Err(_) => MyResponse::item_not_found(),
    Ok(user) => HttpResponse::Ok().json(user),
  }
}

#[post("/users")]
async fn add_user(user: web::Json<User>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let mut user = user.into_inner();
  if let None = user.user_name { return MyResponse::bad_request("user_name not specfied") };
  if let None = user.active { user.active = Some(true) };
  let result = accessor.add_user(user).await;
  match result {
    Err(e) => MyResponse::bad_request(&e.to_string()),
    Ok(_) => MyResponse::ok(),
  }
}

#[delete("/users/{id}")]
async fn delete_user(user_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let user_id = user_id.into_inner();
  let get_result = accessor.get_user(user_id).await;
  if let Err(_) = get_result {
    return MyResponse::item_not_found()
  }
  let result = accessor.delete_user(user_id).await;
  match result {
    Err(_) => MyResponse::item_not_found(),
    Ok(_) => MyResponse::ok(),
  }
}

#[put("/users/{id}")]
async fn update_user(user_id: web::Path<i32>, user: web::Json<User>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let user_id = user_id.into_inner();
  let user = user.into_inner();
  let get_result = accessor.get_user(user_id).await;
  if let Err(_) = get_result {
    return MyResponse::item_not_found()
  }
  if let Some(name) = user.user_name {
    let result = accessor.update_user_name(user_id, &name).await;
    if let Err(e) = result {
      return MyResponse::bad_request(&e.to_string());
    }
  }
  if let Some(activity) = user.active {
    let result = accessor.update_user_activity(user_id, activity).await;
    if let Err(e) = result {
      return MyResponse::bad_request(&e.to_string());
    }
  }
  MyResponse::ok()
}

