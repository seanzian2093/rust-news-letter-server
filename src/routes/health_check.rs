// #![allow(dead_code)]
use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

pub async fn health_check(_req: HttpRequest) -> HttpResponse {
    // `Ok()` returns a Builder instance
    // `finish()` converts the Builder into a Response instance and sends it back to the client
    HttpResponse::Ok().finish()
}
