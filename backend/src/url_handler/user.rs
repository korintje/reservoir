use crate::{auth::Admin, db_handler::DataAccessor, model::User, response::MyResponse};
use actix_web::{delete, get, post, put, web, HttpResponse};
#[get("/users")]
pub async fn get_users(db: web::Data<DataAccessor>) -> HttpResponse {
    match db.get_users().await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[get("/users/{id}")]
pub async fn get_user(db: web::Data<DataAccessor>, id: web::Path<i32>) -> HttpResponse {
    match db.get_user(*id).await {
        Ok(v) => HttpResponse::Ok().json(v),
        Err(sqlx::Error::RowNotFound) => MyResponse::item_not_found(),
        Err(e) => MyResponse::database_error(&e),
    }
}
fn valid(user: &User) -> bool {
    user.user_name
        .as_ref()
        .is_none_or(|v| !v.trim().is_empty() && v.chars().count() <= 200)
}
#[post("/users")]
pub async fn add_user(
    _admin: Admin,
    db: web::Data<DataAccessor>,
    input: web::Json<User>,
) -> HttpResponse {
    let user = input.into_inner();
    if !valid(&user) || user.user_name.is_none() {
        return MyResponse::bad_request("Enter a member name");
    }
    match sqlx::query("INSERT INTO users(user_name,active,sort_order) VALUES(?,?,?)")
        .bind(user.user_name.map(|v| v.trim().to_string()))
        .bind(user.active.unwrap_or(true))
        .bind(user.sort_order.unwrap_or(0))
        .execute(&*db.pool_ref)
        .await
    {
        Ok(v) => HttpResponse::Created().json(serde_json::json!({"id":v.last_insert_rowid()})),
        Err(e) => MyResponse::database_error(&e),
    }
}
#[put("/users/{id}")]
pub async fn update_user(
    _admin: Admin,
    db: web::Data<DataAccessor>,
    id: web::Path<i32>,
    input: web::Json<User>,
) -> HttpResponse {
    let user = input.into_inner();
    if !valid(&user) {
        return MyResponse::bad_request("Check the member name");
    }
    match sqlx::query("UPDATE users SET user_name=COALESCE(?,user_name),active=COALESCE(?,active),sort_order=COALESCE(?,sort_order) WHERE id=?")
        .bind(user.user_name.map(|v|v.trim().to_string())).bind(user.active).bind(user.sort_order).bind(*id)
        .execute(&*db.pool_ref).await {
        Ok(v) if v.rows_affected()==0=>MyResponse::item_not_found(),Ok(_)=>MyResponse::ok(),Err(e)=>MyResponse::database_error(&e)
    }
}
#[delete("/users/{id}")]
pub async fn delete_user(
    _admin: Admin,
    db: web::Data<DataAccessor>,
    id: web::Path<i32>,
) -> HttpResponse {
    match db.delete_user(*id).await {
        Ok(v) if v.rows_affected() == 0 => MyResponse::item_not_found(),
        Ok(_) => MyResponse::ok(),
        Err(e) => MyResponse::database_error(&e),
    }
}
