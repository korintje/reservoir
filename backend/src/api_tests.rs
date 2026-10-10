use crate::{auth, db_handler::DataAccessor, url_handler, utils};
use actix_web::{http::StatusCode, test, web, App};
use chrono::{Duration, Utc};
use serde_json::{json, Value};

#[actix_web::test]
async fn admin_csrf_reservations_conflicts_and_backup_work_together() {
    let _ = env_logger::builder().is_test(true).try_init();
    let directory = tempfile::tempdir().unwrap();
    let db = DataAccessor::new(directory.path().join("reservoir.db").to_str().unwrap()).await;
    let pool = db.pool_ref.clone();
    db.init_table().await;
    let config = utils::Config {
        admin_password_hash: utils::secure_hash("admin-test-password").unwrap(),
        secure_cookies: false,
        ..Default::default()
    };
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .app_data(web::Data::new(auth::Auth::new(config)))
            .service(auth::login)
            .service(auth::logout)
            .service(auth::session)
            .service(url_handler::add_user)
            .service(url_handler::get_users)
            .service(url_handler::add_resource)
            .service(url_handler::update_settings)
            .service(url_handler::get_settings)
            .service(url_handler::add_reservation)
            .service(url_handler::update_reservation)
            .service(url_handler::delete_reservation)
            .service(url_handler::get_reservation)
            .service(url_handler::backup),
    )
    .await;
    let member = json!({"user_name":"Test member","active":true});
    let unauthorized = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/users")
            .set_json(&member)
            .to_request(),
    )
    .await;
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/admin/login")
            .insert_header(("X-Reservoir-Request", "1"))
            .set_json(json!({"password":"admin-test-password"}))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.response().cookies().next().unwrap().into_owned();
    let body: Value = test::read_body_json(response).await;
    let csrf = body["csrf_token"].as_str().unwrap();
    let forbidden = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/users")
            .cookie(cookie.clone())
            .set_json(&member)
            .to_request(),
    )
    .await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let user = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/users")
            .cookie(cookie.clone())
            .insert_header(("X-CSRF-Token", csrf))
            .set_json(&member)
            .to_request(),
    )
    .await;
    assert_eq!(user.status(), StatusCode::CREATED);
    let resource=test::call_service(&app,test::TestRequest::post().uri("/resources").cookie(cookie.clone()).insert_header(("X-CSRF-Token",csrf)).set_json(json!({"resource_name":"Equipment","active":true,"slack_url":"https://example.org/channel"})).to_request()).await;
    assert_eq!(resource.status(), StatusCode::CREATED);
    let start = Utc::now() + Duration::hours(3);
    let end = start + Duration::hours(1);
    let reservation = json!({"resource_id":1,"user_id":1,"start":start,"end":end,"password":"reservation-password","description":"Initial"});
    let added = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/reservations")
            .set_json(&reservation)
            .to_request(),
    )
    .await;
    assert_eq!(added.status(), StatusCode::CREATED);
    let overlap = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/reservations")
            .set_json(&reservation)
            .to_request(),
    )
    .await;
    assert_eq!(overlap.status(), StatusCode::CONFLICT);
    let wrong = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/reservations/1")
            .set_json(json!({"description":"Wrong","password":"wrong","version":1}))
            .to_request(),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
    let update=test::call_service(&app,test::TestRequest::put().uri("/reservations/1").set_json(json!({"start":start+Duration::minutes(30),"password":"reservation-password","version":1})).to_request()).await;
    assert_eq!(update.status(), StatusCode::OK);
    let updated: Value = test::read_body_json(update).await;
    assert_eq!(updated["notification_required"], true);
    assert_eq!(updated["slack_url"], "https://example.org/channel");
    let stale = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/reservations/1")
            .set_json(json!({"description":"Stale","password":"reservation-password","version":1}))
            .to_request(),
    )
    .await;
    assert_eq!(stale.status(), StatusCode::CONFLICT);
    let invalid=test::call_service(&app,test::TestRequest::put().uri("/reservations/1").set_json(json!({"start":end+Duration::hours(1),"description":"Invalid","password":"reservation-password","version":2})).to_request()).await;
    assert_eq!(invalid.status(), StatusCode::CONFLICT);
    let detail: Value = test::call_and_read_body_json(
        &app,
        test::TestRequest::get().uri("/reservations/1").to_request(),
    )
    .await;
    assert_eq!(detail["description"], "Initial");
    assert_eq!(detail["version"], 2);
    let settings = json!({"lab_name":"Test lab","advance_days":30,"slot_minutes":15,"contact_text":"","notification_text":"Notify"});
    let saved = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/settings")
            .cookie(cookie.clone())
            .insert_header(("X-CSRF-Token", csrf))
            .set_json(settings)
            .to_request(),
    )
    .await;
    assert_eq!(saved.status(), StatusCode::OK);
    let downloaded = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/admin/backup")
            .cookie(cookie.clone())
            .to_request(),
    )
    .await;
    assert_eq!(downloaded.status(), StatusCode::OK);
    let bytes = test::read_body(downloaded).await;
    assert!(bytes.starts_with(b"SQLite format 3"));
    let logout = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/admin/logout")
            .cookie(cookie.clone())
            .insert_header(("X-CSRF-Token", csrf))
            .to_request(),
    )
    .await;
    assert_eq!(logout.status(), StatusCode::OK);
    let revoked = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/admin/backup")
            .cookie(cookie)
            .to_request(),
    )
    .await;
    assert_eq!(revoked.status(), StatusCode::UNAUTHORIZED);
    pool.close().await;
}

#[actix_web::test]
async fn migrations_preserve_legacy_records_and_upgrade_password_on_change() {
    let db = DataAccessor::new("sqlite::memory:").await;
    sqlx::raw_sql(include_str!("../migrations/0001_baseline.sql"))
        .execute(&*db.pool_ref)
        .await
        .unwrap();
    sqlx::raw_sql("INSERT INTO users VALUES(1,'Legacy member',1);INSERT INTO resources VALUES(1,'Legacy equipment',1,1,'')").execute(&*db.pool_ref).await.unwrap();
    let start = Utc::now().timestamp() + 172800;
    sqlx::query("INSERT INTO reservations VALUES(1,1,1,?,?,NULL,?,?)")
        .bind(start)
        .bind(start + 3600)
        .bind(utils::hash("legacy-password"))
        .bind(start - 1000)
        .execute(&*db.pool_ref)
        .await
        .unwrap();
    db.migrate().await.unwrap();
    db.migrate().await.unwrap();
    assert_eq!(
        db.get_reservation(1).await.unwrap().user_name,
        "Legacy member"
    );
    let db = web::Data::new(db);
    let app = test::init_service(
        App::new()
            .app_data(db.clone())
            .service(url_handler::update_reservation),
    )
    .await;
    let updated = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/reservations/1")
            .set_json(json!({"password":"legacy-password","description":"Updated","version":1}))
            .to_request(),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    let stored = db.get_passhash_by_id(1).await.unwrap();
    assert!(stored.passhash.unwrap().starts_with("$argon2id$"));
}

#[actix_web::test]
async fn broken_legacy_database_is_rejected_without_migration() {
    let db = DataAccessor::new("sqlite::memory:").await;
    sqlx::raw_sql(include_str!("../migrations/0001_baseline.sql"))
        .execute(&*db.pool_ref)
        .await
        .unwrap();
    sqlx::raw_sql("INSERT INTO reservations VALUES(1,999,999,100,90,NULL,NULL,1)")
        .execute(&*db.pool_ref)
        .await
        .unwrap();
    assert!(db.migrate().await.is_err());
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reservations")
        .fetch_one(&*db.pool_ref)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[actix_web::test]
async fn overlapping_legacy_database_is_rejected_without_removing_records() {
    let db = DataAccessor::new("sqlite::memory:").await;
    sqlx::raw_sql(include_str!("../migrations/0001_baseline.sql"))
        .execute(&*db.pool_ref)
        .await
        .unwrap();
    sqlx::raw_sql("INSERT INTO users VALUES(1,'Member',1);INSERT INTO resources VALUES(1,'Equipment',1,1,'');INSERT INTO reservations VALUES(1,1,1,100,200,NULL,NULL,1);INSERT INTO reservations VALUES(2,1,1,150,250,NULL,NULL,1)")
        .execute(&*db.pool_ref).await.unwrap();
    let error = db.migrate().await.unwrap_err();
    assert!(error.to_string().contains("overlapping"));
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reservations")
        .fetch_one(&*db.pool_ref)
        .await
        .unwrap();
    assert_eq!(count, 2);
}
