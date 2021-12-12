use actix_web::{web, App, HttpServer};
mod model;
mod utils;
mod response;
mod url_handler;
mod db_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_url = utils::get_db_url();
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
    })
    .bind("127.0.0.1:8080")?;
    server.run().await
}
