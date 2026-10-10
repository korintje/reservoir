use crate::{auth::Admin, db_handler::DataAccessor, model::Resource, response::MyResponse};
use actix_web::{delete, get, post, put, web, HttpResponse};
#[get("/resources")]
pub async fn get_resources(db: web::Data<DataAccessor>) -> HttpResponse {
    match db.get_resources().await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[get("/resources/{id}")]
pub async fn get_resource(db: web::Data<DataAccessor>, id: web::Path<i32>) -> HttpResponse {
    match db.get_resource(*id).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(sqlx::Error::RowNotFound) => MyResponse::item_not_found(),
        Err(e) => MyResponse::database_error(&e),
    }
}
fn valid(r: &Resource) -> bool {
    r.resource_name
        .as_ref()
        .is_none_or(|v| !v.trim().is_empty() && v.chars().count() <= 200)
        && r.capacity.is_none_or(|v| (1..=100000).contains(&v))
        && r.custom_field
            .as_ref()
            .is_none_or(|v| v.chars().count() <= 4000)
        && r.slack_channel_id
            .as_ref()
            .is_none_or(|v| v.chars().count() <= 50 && v.chars().all(|c| c.is_ascii_alphanumeric()))
        && r.slack_url.as_ref().is_none_or(|v| {
            v.is_empty()
                || (v.starts_with("https://")
                    && v.chars().count() <= 2000
                    && !v.chars().any(|c| c.is_control()))
        })
}
#[post("/resources")]
pub async fn add_resource(
    _admin: Admin,
    db: web::Data<DataAccessor>,
    input: web::Json<Resource>,
) -> HttpResponse {
    let r = input.into_inner();
    if !valid(&r) || r.resource_name.is_none() {
        return MyResponse::bad_request("Check the resource details");
    }
    match sqlx::query("INSERT INTO resources(resource_name,active,capacity,custom_field,slack_channel_id,slack_url,sort_order) VALUES(?,?,?,?,?,?,?)")
        .bind(r.resource_name.map(|v|v.trim().to_string())).bind(r.active.unwrap_or(true)).bind(r.capacity.unwrap_or(1)).bind(r.custom_field.unwrap_or_default())
        .bind(r.slack_channel_id.unwrap_or_default()).bind(r.slack_url.unwrap_or_default()).bind(r.sort_order.unwrap_or(0))
        .execute(&*db.pool_ref).await {Ok(v)=>HttpResponse::Created().json(serde_json::json!({"id":v.last_insert_rowid()})),Err(e)=>MyResponse::database_error(&e)}
}
#[put("/resources/{id}")]
pub async fn update_resource(
    _admin: Admin,
    db: web::Data<DataAccessor>,
    id: web::Path<i32>,
    input: web::Json<Resource>,
) -> HttpResponse {
    let r = input.into_inner();
    if !valid(&r) {
        return MyResponse::bad_request("Check the resource details");
    }
    match sqlx::query("UPDATE resources SET resource_name=COALESCE(?,resource_name),active=COALESCE(?,active),capacity=COALESCE(?,capacity),custom_field=COALESCE(?,custom_field),slack_channel_id=COALESCE(?,slack_channel_id),slack_url=COALESCE(?,slack_url),sort_order=COALESCE(?,sort_order) WHERE id=?")
        .bind(r.resource_name.map(|v|v.trim().to_string())).bind(r.active).bind(r.capacity).bind(r.custom_field)
        .bind(r.slack_channel_id).bind(r.slack_url).bind(r.sort_order).bind(*id)
        .execute(&*db.pool_ref).await {
        Ok(v) if v.rows_affected()==0=>MyResponse::item_not_found(),Ok(_)=>MyResponse::ok(),Err(e)=>MyResponse::database_error(&e)
    }
}
#[delete("/resources/{id}")]
pub async fn delete_resource(
    _admin: Admin,
    db: web::Data<DataAccessor>,
    id: web::Path<i32>,
) -> HttpResponse {
    match db.delete_resource(*id).await {
        Ok(v) if v.rows_affected() == 0 => MyResponse::item_not_found(),
        Ok(_) => MyResponse::ok(),
        Err(e) => MyResponse::database_error(&e),
    }
}
