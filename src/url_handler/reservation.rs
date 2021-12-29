use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::db_handler::{DataAccessor};
use crate::{utils, response};
use response::{MyResponse};
use crate::model::{ReservationPost, Filter, FullCalendarFilter, FullCalendarEvent, PassWord};

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
  if !reservation.is_valid() {
    return MyResponse::bad_request("'Start time' must be earlier than 'End time'")
  }
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
    accessor: web::Data<DataAccessor>) -> impl Responder {
  let id = reservation_id.into_inner();
  let posted_passhash = utils::hash(&password.password);
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
