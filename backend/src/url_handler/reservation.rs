use crate::{db_handler::DataAccessor, model::*, response::MyResponse, utils};
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::Utc;
use serde_json::json;
#[get("/reservations")]
pub async fn get_reservations(f: web::Query<Filter>, db: web::Data<DataAccessor>) -> HttpResponse {
    if f.from.zip(f.until).is_some_and(|(a, b)| a >= b) {
        return MyResponse::bad_request("Check the search period");
    }
    match db.get_reservations(f.into_inner()).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[get("/reservations/{id}")]
pub async fn get_reservation(id: web::Path<i32>, db: web::Data<DataAccessor>) -> HttpResponse {
    match db.get_reservation(*id).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(sqlx::Error::RowNotFound) => MyResponse::item_not_found(),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[get("/fullcalendar_events")]
pub async fn get_fullcalendar_events(
    f: web::Query<FullCalendarFilter>,
    db: web::Data<DataAccessor>,
) -> HttpResponse {
    match db.get_reservations(f.into_inner().into()).await {
        Ok(v) => HttpResponse::Ok().json(
            v.into_iter()
                .map(FullCalendarEvent::from)
                .collect::<Vec<_>>(),
        ),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[post("/reservations")]
pub async fn add_reservation(
    input: web::Json<ReservationPost>,
    db: web::Data<DataAccessor>,
) -> HttpResponse {
    let r = input.into_inner();
    if !r.is_valid()
        || r.description
            .as_ref()
            .is_some_and(|v| v.chars().count() > 4000)
        || r.password.as_ref().is_some_and(|v| v.chars().count() > 256)
    {
        return MyResponse::bad_request("Check the dates, notes, and password");
    }
    let password = r.password.clone().unwrap_or_default();
    let hash = match web::block(move || utils::secure_hash(&password)).await {
        Ok(Ok(h)) => h,
        _ => return MyResponse::internal_server_error("Password processing failed"),
    };
    match db.add_reservation_hashed(r, hash).await {
        Ok(v) => HttpResponse::Created().json(MutationResult {
            id: v.last_insert_rowid(),
            message: "Booking created".into(),
            notification_required: false,
            notification_reason: None,
            slack_url: None,
        }),
        Err(e) => MyResponse::database_error(&e),
    }
}
async fn authorize(
    db: &DataAccessor,
    id: i32,
    password: Option<String>,
    version: Option<i32>,
) -> Result<PassHash, Box<HttpResponse>> {
    let stored = match db.get_passhash_by_id(id).await {
        Ok(v) => v,
        Err(sqlx::Error::RowNotFound) => return Err(Box::new(MyResponse::item_not_found())),
        Err(e) => return Err(Box::new(MyResponse::database_error(&e))),
    };
    if version.is_some_and(|v| v != stored.version) {
        return Err(Box::new(conflict()));
    }
    let value = stored.passhash.clone();
    let input = password.unwrap_or_default();
    if input.chars().count() > 256 {
        return Err(Box::new(MyResponse::bad_request("Password is too long")));
    }
    match web::block(move || utils::verify_password(&input, value.as_deref())).await {
        Ok(true) => Ok(stored),
        Ok(false) => Err(Box::new(MyResponse::incorrect_password())),
        Err(_) => Err(Box::new(MyResponse::internal_server_error(
            "Password processing failed",
        ))),
    }
}
fn conflict() -> HttpResponse {
    HttpResponse::Conflict()
        .json(json!({"message":"This booking was changed on another device. Reopen its details"}))
}
pub fn notification_reason(
    old: &ReservationReturn,
    start: i64,
    end: i64,
    deleted: bool,
    now: i64,
) -> Option<String> {
    if old.start.timestamp() > now
        && old.start.timestamp() <= now + 86400
        && (deleted || start > old.start.timestamp() || end < old.end.timestamp())
    {
        Some("A booking starting within 24 hours was delayed, shortened, or cancelled".into())
    } else if old.start.timestamp() <= now && old.end.timestamp() > now && end < old.end.timestamp()
    {
        Some("A booking in progress was shortened".into())
    } else {
        None
    }
}
async fn mutate(
    db: &DataAccessor,
    id: i32,
    change: Option<ReservationPut>,
    password: Option<String>,
    version: Option<i32>,
) -> HttpResponse {
    let auth = match authorize(db, id, password.clone(), version).await {
        Ok(v) => v,
        Err(r) => return *r,
    };
    let old = match db.get_reservation(id).await {
        Ok(v) => v,
        Err(e) => return MyResponse::database_error(&e),
    };
    if old.version != auth.version {
        return conflict();
    }
    let replacement = change.as_ref().and_then(|r| r.new_password.clone());
    let needs_upgrade = auth
        .passhash
        .as_ref()
        .is_none_or(|v| !v.starts_with("$argon2"));
    let hashed = if replacement.is_some() || needs_upgrade {
        let value = replacement.unwrap_or_else(|| password.unwrap_or_default());
        if value.chars().count() > 256 {
            return MyResponse::bad_request("Password is too long");
        }
        match web::block(move || utils::secure_hash(&value)).await {
            Ok(Ok(h)) => Some(h),
            _ => return MyResponse::internal_server_error("Password processing failed"),
        }
    } else {
        None
    };
    let start = change
        .as_ref()
        .and_then(|r| r.start)
        .map(|v| v.timestamp())
        .unwrap_or(old.start.timestamp());
    let end = change
        .as_ref()
        .and_then(|r| r.end)
        .map(|v| v.timestamp())
        .unwrap_or(old.end.timestamp());
    let reason = notification_reason(&old, start, end, change.is_none(), Utc::now().timestamp());
    let result:Result<Option<String>,sqlx::Error>=async {
        let mut tx=db.pool_ref.begin().await?;
        let written=if let Some(r)=&change {
            sqlx::query("UPDATE reservations SET user_id=COALESCE(?,user_id),resource_id=COALESCE(?,resource_id),start=COALESCE(?,start),end=COALESCE(?,end),description=COALESCE(?,description),passhash=COALESCE(?,passhash),version=version+1 WHERE id=? AND version=? AND passhash IS ?")
                .bind(r.user_id).bind(r.resource_id).bind(r.start.map(|v|v.timestamp())).bind(r.end.map(|v|v.timestamp())).bind(&r.description).bind(hashed)
                .bind(id).bind(auth.version).bind(&auth.passhash).execute(&mut *tx).await?
        }else{
            sqlx::query("DELETE FROM reservations WHERE id=? AND version=? AND passhash IS ?").bind(id).bind(auth.version).bind(&auth.passhash).execute(&mut *tx).await?
        };
        if written.rows_affected()==0 {return Err(sqlx::Error::Protocol("stale_reservation".into()));}
        let url:String=sqlx::query_scalar("SELECT slack_url FROM resources WHERE id=?").bind(old.resource_id).fetch_one(&mut *tx).await?;
        if let Some(note)=&reason {
            let payload=json!({"before":old,"after":{"start":start,"end":end},"reason":note,"deleted":change.is_none()});
            sqlx::query("INSERT INTO notification_events(id,reservation_id,resource_id,kind,payload,created_at) VALUES(?,?,?,?,?,?)")
                .bind(uuid::Uuid::new_v4().to_string()).bind(id).bind(old.resource_id).bind(if change.is_none(){"cancel"}else{"update"}).bind(payload.to_string()).bind(Utc::now().timestamp()).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(if url.is_empty(){None}else{Some(url)})
    }.await;
    match result {
        Ok(url) => HttpResponse::Ok().json(MutationResult {
            id: id.into(),
            message: if change.is_none() {
                "Booking cancelled".into()
            } else {
                "Booking updated".into()
            },
            notification_required: reason.is_some(),
            notification_reason: reason,
            slack_url: url,
        }),
        Err(sqlx::Error::Protocol(v)) if v == "stale_reservation" => conflict(),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[put("/reservations/{id}")]
pub async fn update_reservation(
    id: web::Path<i32>,
    input: web::Json<ReservationPut>,
    db: web::Data<DataAccessor>,
) -> HttpResponse {
    let r = input.into_inner();
    if r.description
        .as_ref()
        .is_some_and(|v| v.chars().count() > 4000)
    {
        return MyResponse::bad_request("Notes are too long");
    }
    let password = r.password.clone();
    let version = r.version;
    mutate(&db, *id, Some(r), password, version).await
}
#[delete("/reservations/{id}")]
pub async fn delete_reservation(
    id: web::Path<i32>,
    input: web::Json<PassWord>,
    db: web::Data<DataAccessor>,
) -> HttpResponse {
    mutate(&db, *id, None, input.password.clone(), input.version).await
}
