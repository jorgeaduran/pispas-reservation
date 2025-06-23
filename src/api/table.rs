//! # API de Mesas
//!
//! Este módulo maneja todas las operaciones relacionadas con mesas:
//! - Crear nuevas mesas en el plano del restaurante
//! - Listar mesas de un restaurante
//! - Eliminar todas las mesas de un restaurante (clear)
//!
//! Todas las operaciones requieren autenticación mediante token Bearer.

use actix_web::{get, post, delete, web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, oid::ObjectId};
use super::{AppError, AppResult};
use super::restaurant::validate_access_token;
use crate::db::{MongoRepo, Mesa};

/// Estructura para crear una nueva mesa
///
/// Contiene toda la información necesaria para crear una mesa en el plano:
/// posición, dimensiones, capacidad y forma.
#[derive(Deserialize)]
struct NewTable {
    /// ID del restaurante propietario (como string para el frontend)
    id_restaurante: String,
    /// Tipo de elemento (siempre "mesa" por ahora)
    tipo: String,
    /// Nombre único de la mesa dentro del restaurante
    nombre: String,
    /// Posición X en el plano (en píxeles)
    pos_x: f32,
    /// Posición Y en el plano (en píxeles)
    pos_y: f32,
    /// Ancho de la mesa (en píxeles)
    size_x: f32,
    /// Alto de la mesa (en píxeles)
    size_y: f32,
    /// Forma geométrica ("cuadrado" o "circulo")
    forma: String,
    /// Si la mesa acepta reservas
    reservable: bool,
    /// Número mínimo de personas (opcional)
    min_personas: Option<i32>,
    /// Número máximo de personas (opcional)
    max_personas: Option<i32>,
}

/// Estructura de respuesta para una mesa
///
/// Versión simplificada del modelo Mesa para envío al frontend,
/// con ObjectIds convertidos a strings.
#[derive(Serialize)]
struct MesaResponse {
    /// ID único de la mesa (ObjectId convertido a string)
    id: String,
    /// ID del restaurante propietario (ObjectId convertido a string)
    id_restaurante: String,
    /// Tipo de elemento
    tipo: String,
    /// Nombre de la mesa
    nombre: String,
    /// Posición X en el plano
    pos_x: f32,
    /// Posición Y en el plano
    pos_y: f32,
    /// Ancho de la mesa
    size_x: f32,
    /// Alto de la mesa
    size_y: f32,
    /// Forma geométrica
    forma: String,
    /// Si la mesa acepta reservas
    reservable: bool,
    /// Número mínimo de personas
    min_personas: Option<i32>,
    /// Número máximo de personas
    max_personas: Option<i32>,
}

/// Parámetros de consulta para operaciones con mesas
#[derive(Deserialize)]
struct QueryParams {
    /// ID del restaurante
    id_restaurante: String,
}

/// Extrae el token Bearer del header Authorization
///
/// # Parámetros
/// - `req`: Request HTTP que contiene los headers
///
/// # Retorna
/// El token extraído sin el prefijo "Bearer "
///
/// # Errores
/// - `Unauthorized`: Si falta el header, es inválido o no tiene el formato correcto
fn extract_token(req: &HttpRequest) -> AppResult<String> {
    let auth_header = req.headers()
        .get("authorization")
        .ok_or(AppError::Unauthorized("Falta header Authorization".to_string()))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Header Authorization inválido".to_string()))?;

    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::Unauthorized("Formato de token inválido".to_string()));
    }

    Ok(auth_str[7..].to_string())
}

/// Convierte un modelo Mesa interno a la respuesta del API
impl From<Mesa> for MesaResponse {
    fn from(mesa: Mesa) -> Self {
        MesaResponse {
            id: mesa.id.unwrap().to_hex(),
            id_restaurante: mesa.id_restaurante.to_hex(),
            tipo: mesa.tipo,
            nombre: mesa.nombre,
            pos_x: mesa.pos_x,
            pos_y: mesa.pos_y,
            size_x: mesa.size_x,
            size_y: mesa.size_y,
            forma: mesa.forma,
            reservable: mesa.reservable,
            min_personas: mesa.min_personas,
            max_personas: mesa.max_personas,
        }
    }
}

/// Elimina todas las mesas de un restaurante
///
/// **⚠️ Operación destructiva**: Esta función elimina permanentemente todas las mesas
/// del restaurante especificado.
///
/// # Autenticación
/// Requiere token Bearer válido del restaurante propietario.
///
/// # Parámetros
/// - `repo`: Repositorio MongoDB
/// - `query`: ID del restaurante
/// - `req`: Request HTTP con el token de autorización
///
/// # Respuesta
/// ```json
/// {
///   "message": "Se eliminaron 5 mesas correctamente"
/// }
/// ```
///
/// # Errores
/// - `401 Unauthorized`: Token inválido o falta autorización
/// - `403 Forbidden`: No tienes permiso para modificar este restaurante
/// - `500 Internal Server Error`: Error de base de datos
#[delete("/tables/clear")]
async fn clear_tables(
    repo: web::Data<MongoRepo>,
    query: web::Query<QueryParams>,
    req: HttpRequest,
) -> AppResult<impl Responder> {
    let token = extract_token(&req)?;
    let user_id = validate_access_token(repo.get_ref(), &token).await?;

    let id_restaurante = ObjectId::parse_str(&query.id_restaurante)
        .map_err(|_| AppError::Validation("ID de restaurante inválido".to_string()))?;

    // Verificar que el usuario puede acceder a este restaurante
    if user_id != id_restaurante {
        return Err(AppError::Unauthorized("No tienes permiso para modificar este restaurante".to_string()));
    }

    let mesas = repo.mesas();
    let result = mesas
        .delete_many(doc! { "id_restaurante": id_restaurante })
        .await
        .map_err(|e| AppError::Internal(format!("Error eliminando mesas: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Se eliminaron {} mesas correctamente", result.deleted_count)
    })))
}

/// Crea una nueva mesa en el plano del restaurante
///
/// # Autenticación
/// Requiere token Bearer válido del restaurante propietario.
///
/// # Validaciones
/// - El nombre de la mesa no puede estar vacío
/// - La forma debe ser "cuadrado" o "circulo"
/// - Si se especifican min/max personas, min no puede ser mayor que max
/// - No puede existir otra mesa con el mismo nombre en el restaurant
///
/// # Parámetros
/// - `repo`: Repositorio MongoDB
/// - `data`: Datos de la nueva mesa
/// - `req`: Request HTTP con el token de autorización
///
/// # Respuesta
/// ```json
/// {
///   "message": "Mesa creada correctamente",
///   "id": "507f1f77bcf86cd799439011"
/// }
/// ```
///
/// # Errores
/// - `400 Bad Request`: Datos de validación incorrectos
/// - `401 Unauthorized`: Token inválido o falta autorización
/// - `403 Forbidden`: No tienes permiso para crear mesas en este restaurante
/// - `409 Conflict`: Ya existe una mesa con ese nombre
/// - `500 Internal Server Error`: Error de base de datos
#[post("/tables")]
async fn create_table(
    repo: web::Data<MongoRepo>,
    data: web::Json<NewTable>,
    req: HttpRequest,
) -> AppResult<impl Responder> {
    let token = extract_token(&req)?;
    let user_id = validate_access_token(repo.get_ref(), &token).await?;

    let id_restaurante = ObjectId::parse_str(&data.id_restaurante)
        .map_err(|_| AppError::Validation("ID de restaurante inválido".to_string()))?;

    // Verificar que el usuario puede crear mesas para este restaurante
    if user_id != id_restaurante {
        return Err(AppError::Unauthorized("No tienes permiso para crear mesas en este restaurante".to_string()));
    }

    // Validaciones
    if data.nombre.is_empty() {
        return Err(AppError::Validation("El nombre de la mesa es requerido".to_string()));
    }

    if data.forma != "cuadrado" && data.forma != "circulo" {
        return Err(AppError::Validation("La forma debe ser 'cuadrado' o 'circulo'".to_string()));
    }

    if let (Some(min), Some(max)) = (data.min_personas, data.max_personas) {
        if min > max {
            return Err(AppError::Validation("El mínimo de personas no puede ser mayor al máximo".to_string()));
        }
    }

    // Verificar que no exista otra mesa con el mismo nombre en el restaurante
    let mesas = repo.mesas();
    let existing = mesas
        .find_one(doc! {
            "id_restaurante": id_restaurante,
            "nombre": &data.nombre
        })
        .await
        .map_err(|e| AppError::Internal(format!("Error verificando mesa existente: {}", e)))?;

    if existing.is_some() {
        return Err(AppError::Conflict(format!("Ya existe una mesa con el nombre '{}'", data.nombre)));
    }

    let mesa = Mesa {
        id: None,
        id_restaurante,
        tipo: data.tipo.clone(),
        nombre: data.nombre.clone(),
        pos_x: data.pos_x,
        pos_y: data.pos_y,
        size_x: data.size_x,
        size_y: data.size_y,
        forma: data.forma.clone(),
        reservable: data.reservable,
        min_personas: data.min_personas,
        max_personas: data.max_personas,
        created_at: MongoRepo::current_timestamp(),
    };

    let result = mesas
        .insert_one(mesa)
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando mesa: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Mesa creada correctamente",
        "id": result.inserted_id.as_object_id().unwrap().to_hex()
    })))
}

/// Obtiene todas las mesas de un restaurante
///
/// # Autenticación
/// Requiere token Bearer válido del restaurante propietario.
///
/// # Parámetros
/// - `repo`: Repositorio MongoDB
/// - `query`: ID del restaurante
/// - `req`: Request HTTP con el token de autorización
///
/// # Respuesta
/// Lista de mesas con todos sus datos:
/// ```json
/// [
///   {
///     "id": "507f1f77bcf86cd799439011",
///     "id_restaurante": "507f1f77bcf86cd799439012",
///     "tipo": "mesa",
///     "nombre": "Mesa 1",
///     "pos_x": 100.0,
///     "pos_y": 100.0,
///     "size_x": 80.0,
///     "size_y": 80.0,
///     "forma": "cuadrado",
///     "reservable": true,
///     "min_personas": 2,
///     "max_personas": 4
///   }
/// ]
/// ```
///
/// # Errores
/// - `401 Unauthorized`: Token inválido o falta autorización
/// - `403 Forbidden`: No tienes permiso para ver las mesas de este restaurante
/// - `500 Internal Server Error`: Error de base de datos
#[get("/tables")]
async fn get_tables(
    repo: web::Data<MongoRepo>,
    query: web::Query<QueryParams>,
    req: HttpRequest,
) -> AppResult<impl Responder> {
    let token = extract_token(&req)?;
    let user_id = validate_access_token(repo.get_ref(), &token).await?;

    let id_restaurante = ObjectId::parse_str(&query.id_restaurante)
        .map_err(|_| AppError::Validation("ID de restaurante inválido".to_string()))?;

    // Verificar que el usuario puede acceder a este restaurante
    if user_id != id_restaurante {
        return Err(AppError::Unauthorized("No tienes permiso para ver las mesas de este restaurante".to_string()));
    }

    let mesas = repo.mesas();
    let cursor = mesas
        .find(doc! { "id_restaurante": id_restaurante })
        .await
        .map_err(|e| AppError::Internal(format!("Error obteniendo mesas: {}", e)))?;

    let mut results = Vec::new();
    let mut cursor = cursor;

    while cursor.advance().await.map_err(|e| AppError::Internal(format!("Error iterando cursor: {}", e)))? {
        let mesa = cursor.deserialize_current()
            .map_err(|e| AppError::Internal(format!("Error deserializando mesa: {}", e)))?;
        results.push(MesaResponse::from(mesa));
    }

    Ok(HttpResponse::Ok().json(results))
}

/// Configura las rutas relacionadas con mesas
///
/// # Rutas disponibles
/// - `POST /tables` - Crear nueva mesa
/// - `GET /tables` - Listar mesas de un restaurante
/// - `DELETE /tables/clear` - Eliminar todas las mesas
///
/// # Parámetros
/// - `cfg`: Configuración del servicio Actix Web
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_table);
    cfg.service(get_tables);
    cfg.service(clear_tables);
}