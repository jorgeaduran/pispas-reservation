use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize};

#[derive(Deserialize)]
struct MakeReservation {
    access_token: String,
    id_mesa: i32,
    nombre_cliente: String,
    email_cliente: String,
    telefono_cliente: String,
    numero_personas: i32,
    fecha: String,
    hora: String,
}

#[post("/reservations")]
async fn make_reservation(data: web::Json<MakeReservation>) -> impl Responder {
    // Aquí validaríamos access_token y guardaríamos reserva
    HttpResponse::Ok().body("Reserva recibida")
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(make_reservation);
}
