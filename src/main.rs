use actix_web::{web, App, HttpRequest, HttpServer, Result};
use std::path::PathBuf;
use actix_files::NamedFile;

// Import original sturuts
mod models;
mod controllers;
use controllers::*;

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info()
                        .query("filename")
                        .parse()
                        .unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    println!("=== Start reservoir server ===");
    HttpServer::new(|| {
        App::new()
            .service(get_fullcalendar_events)
            .service(post_reservation)
            .service(delete_reservation)
            .service(get_fullcalendar_events)
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
