use crate::{
    auth::{Admin, Auth},
    db_handler::DataAccessor,
    model::Settings,
    response::MyResponse,
};
use actix_web::{get, put, web, HttpResponse};
use serde_json::json;

#[get("/admin/backup")]
pub async fn backup(_admin: Admin, db: web::Data<DataAccessor>) -> HttpResponse {
    let path = std::env::temp_dir().join(format!("reservoir-{}.db", uuid::Uuid::new_v4()));
    let result = sqlx::query("VACUUM INTO ?")
        .bind(path.to_string_lossy().as_ref())
        .execute(&*db.pool_ref)
        .await;
    if let Err(e) = result {
        let _ = std::fs::remove_file(&path);
        return MyResponse::database_error(&e);
    }
    let bytes = web::block(move || {
        let result = std::fs::read(&path);
        let _ = std::fs::remove_file(path);
        result
    })
    .await;
    match bytes {
        Ok(Ok(bytes)) => HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header(("Content-Disposition", "attachment; filename=reservoir.db"))
            .insert_header(("Cache-Control", "no-store"))
            .body(bytes),
        other => {
            log::error!("Backup read failed: {:?}", other);
            MyResponse::internal_server_error("Backup creation failed")
        }
    }
}
#[get("/settings")]
pub async fn get_settings(db: web::Data<DataAccessor>) -> HttpResponse {
    match db.settings().await {
        Ok(s) => HttpResponse::Ok().json(s),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[put("/settings")]
pub async fn update_settings(
    _admin: Admin,
    db: web::Data<DataAccessor>,
    input: web::Json<Settings>,
) -> HttpResponse {
    let s = input.into_inner();
    if s.lab_name.trim().is_empty()
        || s.lab_name.chars().count() > 200
        || !(1..=3650).contains(&s.advance_days)
        || ![15, 30, 60].contains(&s.slot_minutes)
        || s.contact_text.chars().count() > 4000
        || s.notification_text.chars().count() > 4000
    {
        return MyResponse::bad_request("Check the settings");
    }
    let result:Result<(),sqlx::Error>=async {
        let mut tx=db.pool_ref.begin().await?;
        sqlx::query("UPDATE app_settings SET lab_name=?,slot_minutes=?,contact_text=?,notification_text=? WHERE id=1")
            .bind(s.lab_name.trim()).bind(s.slot_minutes).bind(s.contact_text).bind(s.notification_text).execute(&mut *tx).await?;
        sqlx::query("UPDATE reservation_settings SET advance_days=? WHERE id=1").bind(s.advance_days).execute(&mut *tx).await?;
        tx.commit().await
    }.await;
    match result {
        Ok(_) => MyResponse::ok(),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[get("/health")]
pub async fn health(db: web::Data<DataAccessor>) -> HttpResponse {
    match sqlx::query("SELECT 1").execute(&*db.pool_ref).await {
        Ok(_) => {
            HttpResponse::Ok().json(json!({"status":"ok","version":env!("CARGO_PKG_VERSION")}))
        }
        Err(_) => HttpResponse::ServiceUnavailable().json(json!({"status":"unavailable"})),
    }
}
#[get("/admin/maintenance")]
pub async fn maintenance(
    _admin: Admin,
    auth: web::Data<Auth>,
    db: web::Data<DataAccessor>,
) -> HttpResponse {
    let count: Result<i64, _> =
        sqlx::query_scalar("SELECT COUNT(*) FROM notification_events WHERE status='manual'")
            .fetch_one(&*db.pool_ref)
            .await;
    let backup_status = web::block(|| {
        std::fs::read_to_string("/var/lib/reservoir/backup-status.json")
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
    })
    .await
    .ok()
    .flatten();
    match count {
        Ok(count)=>HttpResponse::Ok().json(json!({"version":env!("CARGO_PKG_VERSION"),"basic_auth_enabled":auth.config.basic_auth_enabled,"public_mode":auth.config.public_mode,"manual_notification_count":count,"backup_command":"sudo reservoirctl backup","backup_status":backup_status})),
        Err(e)=>MyResponse::database_error(&e),
    }
}
