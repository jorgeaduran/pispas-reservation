use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use actix_web::delete;
#[derive(Deserialize)]
struct NewTable {
    id_restaurante: i32,
    tipo: String,
    nombre: String,
    pos_x: f32,
    pos_y: f32,
    size_x: f32,
    size_y: f32,
    forma: String,
    reservable: bool,
    min_personas: Option<i32>,
    max_personas: Option<i32>,
}

#[derive(Serialize, sqlx::FromRow)]
struct Mesa {
    id: i32,
    id_restaurante: i32,
    tipo: String,
    nombre: String,
    pos_x: f32,
    pos_y: f32,
    size_x: f32,
    size_y: f32,
    forma: String,
    reservable: bool,
    min_personas: Option<i32>,
    max_personas: Option<i32>,
}

#[derive(Deserialize)]
struct QueryParams {
    id_restaurante: i32,
}
#[delete("/tables/clear")]
async fn clear_tables(
    pool: web::Data<SqlitePool>,
    query: web::Query<QueryParams>,
) -> impl Responder {
    let id_restaurante = query.id_restaurante;

    let result = sqlx::query(
        r#"
        DELETE FROM mesas
        WHERE id_restaurante = ?
        "#,
    )
        .bind(id_restaurante)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Todas las mesas eliminadas correctamente"),
        Err(e) => {
            println!("Error borrando mesas: {:?}", e);
            HttpResponse::InternalServerError().body("Error borrando mesas")
        }
    }
}

#[post("/tables")]
async fn create_table(
    pool: web::Data<SqlitePool>,
    data: web::Json<NewTable>,
) -> impl Responder {
    let result = sqlx::query(
        r#"
    INSERT INTO mesas (id_restaurante, tipo, nombre, pos_x, pos_y, size_x, size_y, forma, reservable, min_personas, max_personas)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#,
    )
        .bind(data.id_restaurante)
        .bind(&data.tipo)
        .bind(&data.nombre)
        .bind(data.pos_x)
        .bind(data.pos_y)
        .bind(data.size_x)
        .bind(data.size_y)
        .bind(&data.forma)
        .bind(data.reservable)
        .bind(data.min_personas)
        .bind(data.max_personas)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Mesa creada correctamente"),
        Err(e) => {
            println!("Error creando mesa: {}", e);
            HttpResponse::InternalServerError().body("Error creando mesa")
        }
    }
}

#[get("/tables")]
async fn get_tables(
    pool: web::Data<SqlitePool>,
    query: web::Query<QueryParams>,
) -> impl Responder {
    let id_restaurante = query.id_restaurante;

    let mesas = sqlx::query_as::<_, Mesa>(
        r#"
        SELECT
            id,
            id_restaurante,
            tipo,
            nombre,
            pos_x,
            pos_y,
            size_x,
            size_y,
            forma,
            reservable,
            min_personas,
            max_personas
        FROM mesas
        WHERE id_restaurante = ?
        "#,
    )
        .bind(id_restaurante)
        .fetch_all(pool.get_ref())
        .await;

    match mesas {
        Ok(mesas) => HttpResponse::Ok().json(mesas),
        Err(e) => {
            println!("Error cargando mesas: {:?}", e);
            HttpResponse::InternalServerError().body("Error cargando mesas")
        }
    }
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_table);
    cfg.service(get_tables);
    cfg.service(clear_tables); // <-- Añadir esta línea
}
