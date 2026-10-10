use super::DataAccessor;
use crate::model::{ReservationPost, ReservationPut};
use chrono::{Duration, Utc};

async fn database() -> DataAccessor {
    let db = DataAccessor::new("sqlite::memory:").await;
    db.init_table().await;
    sqlx::query("INSERT INTO users(id,user_name,active) VALUES (1,'Member',1); INSERT INTO resources(id,resource_name,active,capacity,custom_field) VALUES (1,'Machine',1,1,'')")
        .execute(&*db.pool_ref).await.unwrap();
    db
}

fn reservation(start: i64, end: i64) -> ReservationPost {
    let now = Utc::now();
    ReservationPost {
        user_id: 1,
        resource_id: 1,
        start: now + Duration::seconds(start),
        end: now + Duration::seconds(end),
        description: None,
        password: None,
    }
}

#[actix_web::test]
async fn overlapping_reservations_are_rejected_but_adjacent_are_allowed() {
    let db = database().await;
    let first = reservation(3600, 7200);
    let boundary = first.end;
    db.add_reservation(first).await.unwrap();
    let error = db
        .add_reservation(reservation(4000, 8000))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("reservation_overlap"));
    let mut adjacent = reservation(7200, 10800);
    adjacent.start = boundary;
    db.add_reservation(adjacent).await.unwrap();
}

#[actix_web::test]
async fn invalid_update_is_atomic_and_history_is_preserved() {
    let db = database().await;
    db.add_reservation(reservation(3600, 7200)).await.unwrap();
    let before = db.get_reservation(1).await.unwrap();
    let change = ReservationPut {
        user_id: None,
        resource_id: None,
        start: Some(before.end + Duration::hours(1)),
        end: None,
        description: Some("must not persist".into()),
        password: None,
        new_password: None,
        version: None,
    };
    assert!(db.update_reservation_atomic(1, change).await.is_err());
    let after = db.get_reservation(1).await.unwrap();
    assert_eq!(after.start, before.start);
    assert_eq!(after.description, before.description);
    assert!(db.delete_user(1).await.is_err());
    assert!(db.delete_resource(1).await.is_err());
}

#[actix_web::test]
async fn finished_reservations_and_advance_limit_are_enforced() {
    let db = database().await;
    db.add_reservation(reservation(-7200, -3600)).await.unwrap();
    assert!(db
        .delete_reservation(1)
        .await
        .unwrap_err()
        .to_string()
        .contains("finished_reservation"));
    assert!(db
        .add_reservation(reservation(16 * 86400, 17 * 86400))
        .await
        .unwrap_err()
        .to_string()
        .contains("advance_limit"));
    sqlx::query("UPDATE reservation_settings SET advance_days=30")
        .execute(&*db.pool_ref)
        .await
        .unwrap();
    db.add_reservation(reservation(16 * 86400, 17 * 86400))
        .await
        .unwrap();
}

#[actix_web::test]
async fn simultaneous_overlapping_writes_only_accept_one() {
    let db = database().await;
    let (left, right) = futures_util::future::join(
        db.add_reservation(reservation(3600, 7200)),
        db.add_reservation(reservation(3600, 7200)),
    )
    .await;
    assert_eq!(usize::from(left.is_ok()) + usize::from(right.is_ok()), 1);
}
