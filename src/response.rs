use actix_web::{HttpResponse};
use serde::{Serialize};

#[derive(Serialize)]
pub struct MyResponse {
  pub code: i32,
  pub message: String,
}

impl MyResponse {

  pub fn Ok() -> HttpResponse {
    HttpResponse::Ok().json(
      MyResponse{
        code: 2000, 
        message: "Success".to_string()
      }
    )
  }

  pub fn BadRequest(e: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(
      MyResponse{
        code: 4000, 
        message: e.to_string()
      }
    )
  }

  pub fn Unauthorized(e: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(
      MyResponse{
        code: 4001, 
        message: e.to_string()
      }
    )
  }

  pub fn NotFound(e: &str) -> HttpResponse {
    HttpResponse::NotFound().json(
      MyResponse{
        code: 4004, 
        message: e.to_string()
      }
    )
  }

  pub fn InternalServerError(e: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(
      MyResponse{
        code: 5000, 
        message: e.to_string()
      }
    )
  }

  pub fn ItemNotFound() -> HttpResponse {
    MyResponse::NotFound("Item not found")
  }

  pub fn IncorrectPassword() -> HttpResponse {
    MyResponse::Unauthorized("Incorrect password")
  }

}
