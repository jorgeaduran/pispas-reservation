use actix_web::{post, get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use uuid::Uuid;
use sqlx::Row; // Añade este use arriba
#[derive(Deserialize)]
struct RegisterRestaurant {
    objid_pispas: String,
    name: String,
    password: String,
    confirmar_automaticamente: bool,
}

#[derive(Deserialize)]
struct LoginRequest {
    name: String,
    password: String,
}

#[derive(Serialize)]
struct RestaurantInfo {
    id: i64,
    nombre: String,
    objid_pispas: String,
    confirmar_automaticamente: bool,
}

#[post("/restaurants/register")]
async fn register_restaurant(
    pool: web::Data<SqlitePool>,
    data: web::Json<RegisterRestaurant>,
) -> impl Responder {
    let access_token = Uuid::new_v4().to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO restaurantes (objid_pispas, nombre, password, confirmar_automaticamente, access_token)
        VALUES (?, ?, ?, ?, ?)
        "#
    )
        .bind(&data.objid_pispas)
        .bind(&data.name)
        .bind(&data.password)
        .bind(data.confirmar_automaticamente)
        .bind(&access_token)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({ "access_token": access_token })),
        Err(e) => {
            println!("Error registrando restaurante: {}", e);
            HttpResponse::InternalServerError().body("Error registrando restaurante")
        }
    }
}

#[post("/restaurants/login")]
async fn login_restaurant(
    pool: web::Data<SqlitePool>,
    data: web::Json<LoginRequest>,
) -> impl Responder {
    let result = sqlx::query(
        r#"
        SELECT id, access_token FROM restaurantes
        WHERE nombre = ? AND password = ?
        "#
    )
        .bind(&data.name)
        .bind(&data.password)
        .fetch_optional(pool.get_ref())
        .await;

    match result {
        Ok(Some(record)) => {
            let id = record.try_get::<i64, _>("id").unwrap();
            let access_token = record.try_get::<String, _>("access_token").unwrap();

            HttpResponse::Ok().json(json!({
                "access_token": access_token,
                "id_restaurante": id
            }))
        }
        Ok(None) => HttpResponse::Unauthorized().body("Credenciales incorrectas"),
        Err(e) => {
            println!("Error en login: {}", e);
            HttpResponse::InternalServerError().body("Error en login")
        }
    }
}
#[get("/restaurants/all")]
async fn list_restaurants(
    pool: web::Data<SqlitePool>,
) -> impl Responder {
    let result = sqlx::query(
        r#"
        SELECT id, nombre, objid_pispas, confirmar_automaticamente FROM restaurantes
        "#
    )
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(records) => {
            let restaurantes: Vec<RestaurantInfo> = records.into_iter().map(|r| {
                RestaurantInfo {
                    id: r.try_get("id").unwrap(),
                    nombre: r.try_get("nombre").unwrap(),
                    objid_pispas: r.try_get("objid_pispas").unwrap(),
                    confirmar_automaticamente: r.try_get::<i64, _>("confirmar_automaticamente").unwrap() != 0,
                }
            }).collect();

            HttpResponse::Ok().json(restaurantes)
        }
        Err(e) => {
            println!("Error listando restaurantes: {}", e);
            HttpResponse::InternalServerError().body("Error listando restaurantes")
        }
    }
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register_restaurant);
    cfg.service(login_restaurant);
    cfg.service(list_restaurants);
}
