use actix_web::{web, error, App, HttpServer, HttpResponse};
mod model;
mod utils;
mod response;
mod url_handler;
mod db_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_url = utils::get_db_path();
    let accessor = db_handler::DataAccessor::new(&db_url).await;
    accessor.init_table().await;
    let accessor_state = web::Data::new(accessor);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(accessor_state.clone())
            .service(url_handler::get_resource)
            .service(url_handler::get_resources)
            .service(url_handler::add_resource)
            .service(url_handler::delete_resource)
            .service(url_handler::get_user)
            .service(url_handler::get_users)
            .service(url_handler::add_user)
            .service(url_handler::delete_user)
            .service(url_handler::get_reservation)
            .service(url_handler::get_reservations)
            .service(url_handler::add_reservation)
            .service(url_handler::delete_reservation)
            .service(url_handler::get_fullcalendar_events)
            .app_data(
                web::JsonConfig::default().error_handler(
                    |err, _req| {
                        error::InternalError::from_response(
                            "", 
                            HttpResponse::BadRequest()
                                .content_type("application/json")
                                .body(format!(r#"{{"error":"{}"}}"#, err)),
                        ).into()
                    }
                )
            )
    })
    .bind(utils::get_url())?;
    server.run().await
}
