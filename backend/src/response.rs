use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct MyResponse {
    pub code: i32,
    pub message: String,
}

impl MyResponse {
    pub fn database_error(error: &sqlx::Error) -> HttpResponse {
        if let sqlx::Error::Database(db) = error {
            let message = match db.message() {
                "reservation_overlap" => Some("Bookings for the same resource overlap"),
                "invalid_interval" => Some("End time must be after start time"),
                "inactive_user" => Some("Member does not exist or is inactive"),
                "inactive_resource" => Some("Resource does not exist or is inactive"),
                "advance_limit" => Some("Booking exceeds the allowed horizon"),
                "finished_reservation" => Some("Completed bookings cannot be edited or cancelled"),
                "in_use_reservation" => {
                    Some("Bookings in progress cannot have their start changed or be cancelled")
                }
                "referenced_user" | "referenced_resource" => {
                    Some("Cannot delete an entry with booking history. Deactivate it instead")
                }
                _ => None,
            };
            if let Some(message) = message {
                return HttpResponse::Conflict().json(MyResponse {
                    code: 4009,
                    message: message.into(),
                });
            }
        }
        eprintln!("Database operation failed: {}", error);
        Self::internal_server_error("Database operation failed")
    }

    pub fn ok() -> HttpResponse {
        HttpResponse::Ok().json(MyResponse {
            code: 2000,
            message: "Success".to_string(),
        })
    }

    pub fn bad_request(e: &str) -> HttpResponse {
        HttpResponse::BadRequest().json(MyResponse {
            code: 4000,
            message: e.to_string(),
        })
    }

    pub fn unauthorized(e: &str) -> HttpResponse {
        HttpResponse::Unauthorized().json(MyResponse {
            code: 4001,
            message: e.to_string(),
        })
    }

    pub fn not_found(e: &str) -> HttpResponse {
        HttpResponse::NotFound().json(MyResponse {
            code: 4004,
            message: e.to_string(),
        })
    }

    pub fn internal_server_error(e: &str) -> HttpResponse {
        HttpResponse::InternalServerError().json(MyResponse {
            code: 5000,
            message: e.to_string(),
        })
    }

    pub fn item_not_found() -> HttpResponse {
        MyResponse::not_found("The requested entry was not found")
    }

    pub fn incorrect_password() -> HttpResponse {
        MyResponse::unauthorized("Incorrect booking password")
    }
}
