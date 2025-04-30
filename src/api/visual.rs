use actix_web::{get, HttpResponse, Responder, web};

#[get("/visual")]
async fn get_visual() -> impl Responder {
    HttpResponse::Ok().body("Plano visual en construcción")
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_visual);
}
