use actix_web::{web, App, HttpServer};
mod objects;
mod models;
mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let psql_url = handlers::utils::get_psql_url();
    let accessor = objects::DataAccessor::new(&psql_url).await;
    let accessor_state = web::Data::new(accessor);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(accessor_state.clone())
            .service(handlers::resource::get_resource)
            .service(handlers::resource::add_resource)
    })
    .bind("127.0.0.1:8080")?;
    server.run().await
}
