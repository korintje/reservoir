use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::db_handler::{DataAccessor};
use crate::{utils, response};
use response::{MyResponse};
use crate::model::{Reservation, Filter, FullCalendarFilter, FullCalendarEvent};

#[get("/reservations/{id}")]
async fn get_reservation(reservation_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let result = accessor.get_reservation(reservation_id.into_inner()).await;
  match result {
    Err(_) => MyResponse::ItemNotFound(),
    Ok(item) => HttpResponse::Ok().json(item),
  }
}

#[get("/reservations")]
async fn get_reservations(filter: web::Query<Filter>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let filter = filter.into_inner();
  let result = accessor.get_reservations(filter).await;
  match result {
    Err(_) => MyResponse::ItemNotFound(),
    Ok(item) => HttpResponse::Ok().json(item),
  }
}

#[post("/reservations")]
async fn add_reservation(reservation: web::Json<Reservation>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let reservation = reservation.into_inner();
  let result = accessor.add_reservation(reservation).await;
  match result {
    Err(e) => MyResponse::BadRequest(&e.to_string()),
    Ok(_) => MyResponse::Ok(),
  }
}

#[delete("/reservations/{id}")]
async fn delete_reservation(reservation_id: web::Path<i32>, reservation: web::Json<Reservation>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let id = reservation_id.into_inner();
  let posted_passhash = utils::hash(&reservation.password);
  let get_result = accessor.get_passhash_by_id(id).await;
  if let Err(_) = get_result {
    return MyResponse::ItemNotFound()
  }
  if let Some(stored_passhash) = get_result.unwrap().passhash {
    if stored_passhash != posted_passhash {
      return MyResponse::IncorrectPassword()
    }
  }
  let del_result = accessor.delete_reservation(id).await;
  match del_result {
    Err(e) => MyResponse::InternalServerError(&e.to_string()),
    Ok(_) => MyResponse::Ok(),
  }
}

#[get("/fullcalendar_events")]
async fn get_fullcalendar_events(filter: web::Query<FullCalendarFilter>, accessor: web::Data<DataAccessor>) -> HttpResponse {
  let filter = filter.into_inner();
  let filter = Filter{from: filter.start, until: filter.end, resource_id: filter.resource_id, user_id: None};
  let result = accessor.get_reservations(filter).await;
  match result {
    Err(_) => MyResponse::ItemNotFound(),
    Ok(reservations) => {
      let events: Vec<FullCalendarEvent> = reservations.into_iter().map(|rsv| FullCalendarEvent::from(rsv)).collect();
      HttpResponse::Ok().json(events)
    } 
  }
}
