use actix_web::{get, post, delete, put, web, HttpResponse, Responder};
use crate::db_handler::{DataAccessor};
use crate::{utils, response};
use response::{MyResponse};
use crate::model::{
  ReservationPost, ReservationPut, Filter, 
  FullCalendarFilter, FullCalendarEvent, PassWord
};

#[get("/reservations/{id}")]
async fn get_reservation(reservation_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let result = accessor.get_reservation(reservation_id.into_inner()).await;
  match result {
    Err(_) => MyResponse::item_not_found(),
    Ok(item) => HttpResponse::Ok().json(item),
  }
}

#[get("/reservations")]
async fn get_reservations(filter: web::Query<Filter>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let filter = filter.into_inner();
  let result = accessor.get_reservations(filter).await;
  match result {
    Err(_) => MyResponse::item_not_found(),
    Ok(item) => HttpResponse::Ok().json(item),
  }
}

#[post("/reservations")]
async fn add_reservation(reservation: web::Json<ReservationPost>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let reservation = reservation.into_inner();
  let user_id = reservation.user_id;
  let resource_id = reservation.resource_id;
  if let Err(_) = accessor.get_user(user_id).await {
    return MyResponse::not_found(&format!("User {} not found", user_id))
  }
  if let Err(_) = accessor.get_resource(resource_id).await {
    return MyResponse::not_found(&format!("Resource {} not found", resource_id))
  }
  let result = accessor.add_reservation(reservation).await;
  match result {
    Err(e) => MyResponse::bad_request(&e.to_string()),
    Ok(_) => MyResponse::ok(),
  }
}

#[delete("/reservations/{id}")]
async fn delete_reservation(
  reservation_id: web::Path<i32>, 
  password: web::Json<PassWord>, 
  accessor: web::Data<DataAccessor>
)
-> impl Responder {
  let id = reservation_id.into_inner();
  let posted_passhash = utils::hash_anyway(&password.password);
  let get_result = accessor.get_passhash_by_id(id).await;
  if let Err(_) = get_result {
    return MyResponse::item_not_found()
  }
  if let Some(stored_passhash) = get_result.unwrap().passhash {
    if stored_passhash != posted_passhash {
      return MyResponse::incorrect_password()
    }
  }
  let del_result = accessor.delete_reservation(id).await;
  match del_result {
    Err(e) => MyResponse::internal_server_error(&e.to_string()),
    Ok(_) => MyResponse::ok(),
  }
}

#[put("/reservations/{id}")]
async fn update_reservation(
  reservation_id: web::Path<i32>,
  reservation: web::Json<ReservationPut>,
  accessor: web::Data<DataAccessor>
)
-> impl Responder {
  let id = reservation_id.into_inner();
  let posted_passhash = utils::hash_anyway(&reservation.password);
  let get_result = accessor.get_passhash_by_id(id).await;
  if let Err(_) = get_result {
    return MyResponse::item_not_found()
  }
  if let Some(stored_passhash) = get_result.unwrap().passhash {
    if stored_passhash != posted_passhash {
      return MyResponse::incorrect_password()
    }
  }
  let reservation = reservation.into_inner();
  let mut results = vec![];
  if let Some(user_id) = reservation.user_id {
    let result = accessor.update_user(id, user_id).await;
    results.push(result);
  }
  if let Some(resource_id) = reservation.resource_id {
    let result = accessor.update_resource(id, resource_id).await;
    results.push(result);
  }
  if let Some(start) = reservation.start {
    let result = accessor.update_start(id, start.timestamp()).await;
    results.push(result);
  }
  if let Some(end) = reservation.end {
    let result = accessor.update_end(id, end.timestamp()).await;
    results.push(result);
  }
  if let Some(description) = reservation.description {
    let result = accessor.update_description(id, &description).await;
    results.push(result);
  }
  if let Some(new_password) = reservation.new_password {
    let new_passhash = utils::hash(&new_password);
    let result = accessor.update_passhash(id, &new_passhash).await;
    results.push(result);
  }
  let mut errors: Vec<String> = vec![];
  for result in results.iter() {
    if let Err(e) = result {
      errors.push(e.to_string())
    }
  }
  if !errors.is_empty() {
    return MyResponse::bad_request(&errors.join("\n"));
  }
  MyResponse::ok()
}

#[get("/fullcalendar_events")]
async fn get_fullcalendar_events(fc_filter: web::Query<FullCalendarFilter>, accessor: web::Data<DataAccessor>) -> HttpResponse {
  let filter = Filter::from(fc_filter.into_inner());
  let result = accessor.get_reservations(filter).await;
  match result {
    Err(_) => MyResponse::item_not_found(),
    Ok(reservations) => {
      let events: Vec<FullCalendarEvent> = reservations.into_iter().map(|rsv| FullCalendarEvent::from(rsv)).collect();
      HttpResponse::Ok().json(events)
    } 
  }
}
