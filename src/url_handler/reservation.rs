use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::db_handler::{DataAccessor};
use crate::utils;
use crate::models::{Reservation, Filter, MyError, MySuccess, FullCalendarFilter, FullCalendarEvent};

#[get("/reservations/{id}")]
async fn get_reservation(reservation_id: web::Path<i32>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let result = accessor.get_reservation(reservation_id.into_inner()).await;
  match result {
    Err(e) => HttpResponse::NotFound().json(MyError{code: 4004, message: e.to_string()}),
    Ok(reservation) => HttpResponse::Ok().json(reservation),
  }
}

#[get("/reservations")]
async fn get_reservations(filter: web::Query<Filter>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let filter = filter.into_inner();
  let result = accessor.get_reservations(filter).await;
  match result {
    Err(e) => HttpResponse::NotFound().json(MyError{code: 4000, message: e.to_string()}),
    Ok(reservations) => HttpResponse::Ok().json(reservations),
  }
}

#[post("/reservations")]
async fn add_reservation(reservation: web::Json<Reservation>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let reservation = reservation.into_inner();
  let result = accessor.add_reservation(reservation).await;
  match result {
    Err(e) => HttpResponse::InternalServerError().json(MyError{code: 400, message: e.to_string()}),
    Ok(_) => HttpResponse::Ok().json(MySuccess{code: 2000, message: "Reservation has successfully added".to_string()}),
  }
}

#[delete("/reservations/{id}")]
async fn delete_reservation(reservation_id: web::Path<i32>, reservation: web::Json<Reservation>, accessor: web::Data<DataAccessor>) -> impl Responder {
  let id = reservation_id.into_inner();
  let posted_passhash = utils::hash(&reservation.password);
  let get_result = accessor.get_passhash_by_id(id).await;
  match get_result {
    Err(e) => HttpResponse::NotFound().json(MyError{code: 4000, message: e.to_string()}),
    Ok(ph) => {
      let stored_passhash = utils::hash(&ph.passhash);
      if stored_passhash == posted_passhash {
        let del_result = accessor.delete_reservation(id).await;
        match del_result {
          Err(e) => HttpResponse::Unauthorized().json(MyError{code: 4003, message: e.to_string()}),
          Ok(_) => HttpResponse::Ok().json(MySuccess{code: 2000, message: format!("Reservation {} has successfully added", id)})
        }
      }else{
        HttpResponse::Unauthorized().json(MyError{code: 4003, message: "Invalid password".to_string()})
      }
    }
  }
}

#[get("/fullcalendar_events")]
async fn get_fullcalendar_events(filter: web::Query<FullCalendarFilter>, accessor: web::Data<DataAccessor>) -> HttpResponse {
  let filter = filter.into_inner();
  let filter = Filter{from: filter.start, until: filter.end, resource_id: filter.resource_id, user_id: None};
  let result = accessor.get_reservations(filter).await;
  match result {
    Err(e) => HttpResponse::NotFound().json(MyError{code: 4000, message: e.to_string()}),
    Ok(reservations) => {
      let events: Vec<FullCalendarEvent> = reservations.into_iter().map(|rsv| FullCalendarEvent::from_reservation(rsv)).collect();
      HttpResponse::Ok().json(events)
    } 
  }
}
