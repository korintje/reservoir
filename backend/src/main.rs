use actix_web::{middleware::Logger, web, App, HttpServer};
use std::io;
#[cfg(test)]
mod api_tests;
mod auth;
mod db_handler;
mod model;
mod response;
mod url_handler;
mod utils;
#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("hash-password") {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let password = input.trim_end_matches(['\r', '\n']);
        if !(8..=256).contains(&password.chars().count()) {
            return Err(io::Error::other("Password must have 8 to 256 characters"));
        }
        println!(
            "{}",
            utils::secure_hash(password).map_err(io::Error::other)?
        );
        return Ok(());
    }
    let config = utils::Config::load()?;
    if config.admin_password_hash.is_empty() {
        return Err(io::Error::other(
            "Set admin_password_hash in the config; use reservoir hash-password",
        ));
    }
    let accessor = db_handler::DataAccessor::connect(&config.db_path)
        .await
        .map_err(io::Error::other)?;
    accessor.migrate().await.map_err(io::Error::other)?;
    let auth = web::Data::new(auth::Auth::new(config.clone()));
    let db = web::Data::new(accessor);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %r %s %D"))
            .app_data(db.clone())
            .app_data(auth.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(32768)
                    .error_handler(|_, _| {
                        actix_web::error::ErrorBadRequest("Check the JSON input")
                    }),
            )
            .service(
                web::scope("/api")
                    .service(auth::login)
                    .service(auth::session)
                    .service(auth::logout)
                    .service(url_handler::get_users)
                    .service(url_handler::get_user)
                    .service(url_handler::add_user)
                    .service(url_handler::update_user)
                    .service(url_handler::delete_user)
                    .service(url_handler::get_resources)
                    .service(url_handler::get_resource)
                    .service(url_handler::add_resource)
                    .service(url_handler::update_resource)
                    .service(url_handler::delete_resource)
                    .service(url_handler::get_reservations)
                    .service(url_handler::get_reservation)
                    .service(url_handler::add_reservation)
                    .service(url_handler::update_reservation)
                    .service(url_handler::delete_reservation)
                    .service(url_handler::get_fullcalendar_events)
                    .service(url_handler::get_settings)
                    .service(url_handler::update_settings)
                    .service(url_handler::health)
                    .service(url_handler::maintenance)
                    .service(url_handler::backup),
            )
    })
    .workers(2)
    .worker_max_blocking_threads(4)
    .bind(&config.web_url)?
    .run()
    .await
}
