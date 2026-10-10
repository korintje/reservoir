use crate::utils::Config;
use actix_web::{
    cookie::{time::Duration as CookieDuration, Cookie, SameSite},
    dev::Payload,
    error, get, post, web, FromRequest, HttpRequest, HttpResponse,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::{HashMap, VecDeque},
    future::{ready, Ready},
    sync::Mutex,
    time::{Duration, Instant},
};
struct AdminSession {
    csrf: String,
    expires: Instant,
}
pub struct Auth {
    pub config: Config,
    sessions: Mutex<HashMap<String, AdminSession>>,
    attempts: Mutex<VecDeque<Instant>>,
}
impl Auth {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            sessions: Mutex::new(HashMap::new()),
            attempts: Mutex::new(VecDeque::new()),
        }
    }
    fn csrf(&self, req: &HttpRequest) -> Option<String> {
        let cookie = req.cookie("reservoir_admin")?;
        let mut sessions = self.sessions.lock().ok()?;
        sessions.retain(|_, s| s.expires > Instant::now());
        sessions.get(cookie.value()).map(|s| s.csrf.clone())
    }
}
pub struct Admin;
impl FromRequest for Admin {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let result = req
            .app_data::<web::Data<Auth>>()
            .and_then(|a| a.csrf(req))
            .ok_or_else(|| error::ErrorUnauthorized("Administrator login is required"))
            .and_then(|csrf| {
                if req.method().is_safe()
                    || req
                        .headers()
                        .get("X-CSRF-Token")
                        .and_then(|v| v.to_str().ok())
                        == Some(csrf.as_str())
                {
                    Ok(Admin)
                } else {
                    Err(error::ErrorForbidden("Check your session and log in again"))
                }
            });
        ready(result)
    }
}
#[derive(Deserialize)]
pub struct Login {
    password: String,
}
#[post("/admin/login")]
pub async fn login(
    req: HttpRequest,
    input: web::Json<Login>,
    auth: web::Data<Auth>,
) -> actix_web::Result<HttpResponse> {
    if req
        .headers()
        .get("X-Reservoir-Request")
        .and_then(|v| v.to_str().ok())
        != Some("1")
    {
        return Err(error::ErrorForbidden("Invalid request"));
    }
    {
        let mut attempts = auth
            .attempts
            .lock()
            .map_err(|_| error::ErrorInternalServerError("Authentication unavailable"))?;
        attempts.retain(|t| t.elapsed() < Duration::from_secs(60));
        if attempts.len() >= 10 {
            return Ok(HttpResponse::TooManyRequests()
                .json(json!({"message":"Please wait and try again"})));
        }
        attempts.push_back(Instant::now());
    }
    let stored = auth.config.admin_password_hash.clone();
    let password = input.into_inner().password;
    let valid = web::block(move || crate::utils::verify_password(&password, Some(&stored)))
        .await
        .map_err(|_| error::ErrorInternalServerError("Authentication unavailable"))?;
    if !valid {
        return Ok(HttpResponse::Unauthorized()
            .json(json!({"message":"Incorrect administrator password"})));
    }
    let token = uuid::Uuid::new_v4().to_string();
    let csrf = uuid::Uuid::new_v4().to_string();
    {
        let mut sessions = auth
            .sessions
            .lock()
            .map_err(|_| error::ErrorInternalServerError("Authentication unavailable"))?;
        sessions.retain(|_, s| s.expires > Instant::now());
        if sessions.len() >= 100 {
            return Err(error::ErrorServiceUnavailable("Session limit"));
        }
        sessions.insert(
            token.clone(),
            AdminSession {
                csrf: csrf.clone(),
                expires: Instant::now() + Duration::from_secs(8 * 3600),
            },
        );
    }
    let cookie = Cookie::build("reservoir_admin", token)
        .path("/api")
        .http_only(true)
        .secure(auth.config.secure_cookies)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::hours(8))
        .finish();
    Ok(HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"csrf_token":csrf})))
}
#[get("/admin/session")]
pub async fn session(req: HttpRequest, auth: web::Data<Auth>) -> HttpResponse {
    match auth.csrf(&req) {
        Some(csrf) => HttpResponse::Ok().json(json!({"csrf_token":csrf})),
        None => {
            HttpResponse::Unauthorized().json(json!({"message":"Administrator login is required"}))
        }
    }
}
#[post("/admin/logout")]
pub async fn logout(_admin: Admin, req: HttpRequest, auth: web::Data<Auth>) -> HttpResponse {
    if let Some(cookie) = req.cookie("reservoir_admin") {
        if let Ok(mut sessions) = auth.sessions.lock() {
            sessions.remove(cookie.value());
        }
    }
    let cookie = Cookie::build("reservoir_admin", "")
        .path("/api")
        .http_only(true)
        .secure(auth.config.secure_cookies)
        .same_site(SameSite::Strict)
        .max_age(CookieDuration::ZERO)
        .finish();
    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"message":"Logged out"}))
}
