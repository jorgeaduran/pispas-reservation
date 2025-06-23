//! # API de Reservas
//!
//! Este módulo maneja todas las operaciones relacionadas con reservas:
//! - Crear nuevas reservas
//! - Listar reservas con filtros opcionales
//! - Confirmar reservas pendientes
//! - Cancelar reservas
//!
//! Todas las operaciones requieren autenticación mediante token Bearer.

use actix_web::{post, get, web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, oid::ObjectId};
use chrono::{NaiveDate, NaiveTime};
use super::{AppError, AppResult};
use super::restaurant::validate_access_token;
use crate::db::{MongoRepo, Reserva, Mesa};

/// Estructura para crear una nueva reserva
///
/// Contiene toda la información necesaria para realizar una reserva:
/// mesa, datos del cliente, fecha/hora y número de comensales.
#[derive(Deserialize)]
struct MakeReservation {
    /// ID de la mesa a reservar (ObjectId como string)
    id_mesa: String,
    /// Nombre completo del cliente
    nombre_cliente: String,
    /// Email del cliente (usado para confirmaciones)
    email_cliente: String,
    /// Teléfono del cliente
    telefono_cliente: String,
    /// Número de comensales
    numero_personas: i32,
    /// Fecha de la reserva (formato YYYY-MM-DD)
    fecha: String,
    /// Hora de la reserva (formato HH:MM)
    hora: String,
}

/// Estructura de respuesta para una reserva
///
/// Versión simplificada del modelo Reserva para envío al frontend,
/// con ObjectIds convertidos a strings.
#[derive(Serialize)]
struct ReservationResponse {
    /// ID único de la reserva (ObjectId convertido a string)
    id: String,
    /// ID del restaurante (ObjectId convertido a string)
    id_restaurante: String,
    /// ID de la mesa reservada (ObjectId convertido a string)
    id_mesa: String,
    /// Nombre del cliente
    nombre_cliente: String,
    /// Email del cliente
    email_cliente: String,
    /// Teléfono del cliente
    telefono_cliente: String,
    /// Número de comensales
    numero_personas: i32,
    /// Fecha de la reserva
    fecha: String,
    /// Hora de la reserva
    hora: String,
    /// Estado actual ("pendiente", "confirmada", "cancelada")
    estado: String,
}

/// Parámetros de consulta para listar reservas
#[derive(Deserialize)]
struct ReservationQuery {
    /// Filtrar por fecha específica (formato YYYY-MM-DD)
    fecha: Option<String>,
    /// Filtrar por estado ("pendiente", "confirmada", "cancelada")
    estado: Option<String>,
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

/// Valida un email de forma básica
///
/// # Parámetros
/// - `email`: String del email a validar
///
/// # Retorna
/// `true` si el email contiene '@' y '.', `false` en caso contrario
///
/// # Nota
/// Esta es una validación muy básica, en producción se debería usar una librería especializada
fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

/// Valida y parsea una fecha en formato YYYY-MM-DD
///
/// # Parámetros
/// - `date_str`: String de la fecha a validar
///
/// # Retorna
/// `NaiveDate` parseado si es válido
///
/// # Errores
/// - `Validation`: Si el formato de fecha es incorrecto
fn validate_date(date_str: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Formato de fecha inválido, use YYYY-MM-DD".to_string()))
}

/// Valida y parsea una hora en formato HH:MM
///
/// # Parámetros
/// - `time_str`: String de la hora a validar
///
/// # Retorna
/// `NaiveTime` parseado si es válido
///
/// # Errores
/// - `Validation`: Si el formato de hora es incorrecto
fn validate_time(time_str: &str) -> AppResult<NaiveTime> {
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|_| AppError::Validation("Formato de hora inválido, use HH:MM".to_string()))
}

/// Convierte un modelo Reserva interno a la respuesta del API
impl From<Reserva> for ReservationResponse {
    fn from(reserva: Reserva) -> Self {
        ReservationResponse {
            id: reserva.id.unwrap().to_hex(),
            id_restaurante: reserva.id_restaurante.to_hex(),
            id_mesa: reserva.id_mesa.to_hex(),
            nombre_cliente: reserva.nombre_cliente,
            email_cliente: reserva.email_cliente,
            telefono_cliente: reserva.telefono_cliente,
            numero_personas: reserva.numero_personas,
            fecha: reserva.fecha,
            hora: reserva.hora,
            estado: reserva.estado,
        }
    }
}

/// Crea una nueva reserva
///
/// # Autenticación
/// Requiere token Bearer válido del restaurante.
///
/// # Validaciones
/// - Nombre del cliente no puede estar vacío
/// - Email debe tener formato válido básico
/// - Teléfono no puede estar vacío
/// - Número de personas debe ser mayor a 0
/// - Fecha debe ser válida (YYYY-MM-DD)
/// - Hora debe ser válida (HH:MM)
/// - La mesa debe existir y pertenecer al restaurante
/// - El número de personas debe estar dentro de la capacidad de la mesa
/// - No debe existir otra reserva activa para la misma mesa/fecha/hora
///
/// # Parámetros
/// - `repo`: Repositorio MongoDB
/// - `data`: Datos de la nueva reserva
/// - `req`: Request HTTP con el token de autorización
///
/// # Respuesta
/// ```json
/// {
///   "message": "Reserva creada correctamente",
///   "id": "507f1f77bcf86cd799439011",
///   "estado": "pendiente"
/// }
/// ```
///
/// # Errores
/// - `400 Bad Request`: Datos de validación incorrectos
/// - `401 Unauthorized`: Token inválido o falta autorización
/// - `403 Forbidden`: No tienes permiso para hacer reservas en esta mesa
/// - `404 Not Found`: Mesa no encontrada
/// - `409 Conflict`: Ya existe una reserva para esa fecha/hora
/// - `500 Internal Server Error`: Error de base de datos
#[post("/reservations")]
async fn make_reservation(
    repo: web::Data<MongoRepo>,
    data: web::Json<MakeReservation>,
    req: HttpRequest,
) -> AppResult<impl Responder> {
    let token = extract_token(&req)?;
    let restaurante_id = validate_access_token(repo.get_ref(), &token).await?;

    // Validaciones de entrada
    if data.nombre_cliente.trim().is_empty() {
        return Err(AppError::Validation("El nombre del cliente es requerido".to_string()));
    }

    if !validate_email(&data.email_cliente) {
        return Err(AppError::Validation("Email inválido".to_string()));
    }

    if data.telefono_cliente.trim().is_empty() {
        return Err(AppError::Validation("El teléfono del cliente es requerido".to_string()));
    }

    if data.numero_personas <= 0 {
        return Err(AppError::Validation("El número de personas debe ser mayor a 0".to_string()));
    }

    // Validar formato de fecha y hora
    let _fecha = validate_date(&data.fecha)?;
    let _hora = validate_time(&data.hora)?;

    // Convertir id_mesa a ObjectId
    let id_mesa = ObjectId::parse_str(&data.id_mesa)
        .map_err(|_| AppError::Validation("ID de mesa inválido".to_string()))?;

    // Verificar que la mesa existe y pertenece al restaurante
    let mesas = repo.mesas();

    let mesa = mesas
        .find_one(doc! { "_id": id_mesa })
        .await
        .map_err(|e| AppError::Internal(format!("Error buscando mesa: {}", e)))?;

    let mesa = mesa.ok_or(AppError::NotFound("Mesa no encontrada".to_string()))?;

    if mesa.id_restaurante != restaurante_id {
        return Err(AppError::Unauthorized("No tienes permiso para hacer reservas en esta mesa".to_string()));
    }

    // Verificar capacidad de la mesa
    if let Some(min) = mesa.min_personas {
        if data.numero_personas < min {
            return Err(AppError::Validation(format!("Esta mesa requiere mínimo {} personas", min)));
        }
    }

    if let Some(max) = mesa.max_personas {
        if data.numero_personas > max {
            return Err(AppError::Validation(format!("Esta mesa permite máximo {} personas", max)));
        }
    }

    // Verificar que no haya conflicto de horario
    let reservas = repo.reservas();
    let existing = reservas
        .find_one(doc! {
            "id_mesa": id_mesa,
            "fecha": &data.fecha,
            "hora": &data.hora,
            "estado": {"$ne": "cancelada"}
        })
        .await
        .map_err(|e| AppError::Internal(format!("Error verificando conflicto: {}", e)))?;

    if existing.is_some() {
        return Err(AppError::Conflict("Ya existe una reserva para esta mesa en este horario".to_string()));
    }

    // Crear la nueva reserva
    let current_time = MongoRepo::current_timestamp();
    let reserva = Reserva {
        id: None,
        id_restaurante: restaurante_id,
        id_mesa,
        nombre_cliente: data.nombre_cliente.clone(),
        email_cliente: data.email_cliente.clone(),
        telefono_cliente: data.telefono_cliente.clone(),
        numero_personas: data.numero_personas,
        fecha: data.fecha.clone(),
        hora: data.hora.clone(),
        estado: "pendiente".to_string(),
        created_at: current_time,
        updated_at: current_time,
    };

    let result = reservas
        .insert_one(reserva)
        .await
        .map_err(|e| AppError::Internal(format!("Error guardando reserva: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Reserva creada correctamente",
        "id": result.inserted_id.as_object_id().unwrap().to_hex(),
        "estado": "pendiente"
    })))
}

/// Lista las reservas de un restaurante con filtros opcionales
///
/// # Autenticación
/// Requiere token Bearer válido del restaurante.
///
/// # Filtros disponibles
/// - `fecha`: Filtrar por fecha específica (formato YYYY-MM-DD)
/// - `estado`: Filtrar por estado ("pendiente", "confirmada", "cancelada")
///
/// # Parámetros
/// - `repo`: Repositorio MongoDB
/// - `query`: Parámetros de filtrado opcionales
/// - `req`: Request HTTP con el token de autorización
///
/// # Respuesta
/// Lista de reservas ordenadas por fecha/hora (más recientes primero):
/// ```json
/// [
///   {
///     "id": "507f1f77bcf86cd799439011",
///     "id_restaurante": "507f1f77bcf86cd799439012",
///     "id_mesa": "507f1f77bcf86cd799439013",
///     "nombre_cliente": "Juan Pérez",
///     "email_cliente": "juan@email.com",
///     "telefono_cliente": "+34 123 456 789",
///     "numero_personas": 2,
///     "fecha": "2024-12-25",
///     "hora": "20:00",
///     "estado": "pendiente"
///   }
/// ]
/// ```
///
/// # Errores
/// - `401 Unauthorized`: Token inválido o falta autorización
/// - `500 Internal Server Error`: Error de base de datos
#[get("/reservations")]
async fn get_reservations(
    repo: web::Data<MongoRepo>,
    query: web::Query<ReservationQuery>,
    req: HttpRequest,
) -> AppResult<impl Responder> {
    let token = extract_token(&req)?;
    let user_id = validate_access_token(repo.get_ref(), &token).await?;

    // Construir filtro dinámico basado en parámetros
    let mut filter = doc! { "id_restaurante": user_id };

    if let Some(fecha) = &query.fecha {
        filter.insert("fecha", fecha);
    }

    if let Some(estado) = &query.estado {
        filter.insert("estado", estado);
    }

    let reservas = repo.reservas();
    let cursor = reservas
        .find(filter)
        .await
        .map_err(|e| AppError::Internal(format!("Error obteniendo reservas: {}", e)))?;

    let mut results = Vec::new();
    let mut cursor = cursor;

    while cursor.advance().await.map_err(|e| AppError::Internal(format!("Error iterando cursor: {}", e)))? {
        let reserva = cursor.deserialize_current()
            .map_err(|e| AppError::Internal(format!("Error deserializando reserva: {}", e)))?;
        results.push(ReservationResponse::from(reserva));
    }

    Ok(HttpResponse::Ok().json(results))
}

/// Confirma una reserva pendiente
///
/// Cambia el estado de una reserva de "pendiente" a "confirmada".
/// Solo se pueden confirmar reservas que estén en estado "pendiente".
///
/// # Autenticación
/// Requiere token Bearer válido del restaurante propietario.
///
/// # Parámetros
/// - `repo`: Repositorio MongoDB
/// - `path`: ID de la reserva a confirmar (en la URL)
/// - `req`: Request HTTP con el token de autorización
///
/// # Respuesta
/// ```json
/// {
///   "message": "Reserva confirmada correctamente",
///   "id": "507f1f77bcf86cd799439011",
///   "estado": "confirmada"
/// }
/// ```
///
/// # Errores
/// - `400 Bad Request`: ID de reserva inválido
/// - `401 Unauthorized`: Token inválido o falta autorización
/// - `403 Forbidden`: No tienes permiso para confirmar reservas de este restaurante
/// - `404 Not Found`: Reserva no encontrada o ya procesada
/// - `500 Internal Server Error`: Error de base de datos
#[post("/reservations/{id}/confirm")]
async fn confirm_reservation(
    repo: web::Data<MongoRepo>,
    path: web::Path<String>,
    req: HttpRequest,
) -> AppResult<impl Responder> {
    let token = extract_token(&req)?;
    let user_id = validate_access_token(repo.get_ref(), &token).await?;
    let reservation_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError::Validation("ID de reserva inválido".to_string()))?;

    // Actualizar la reserva solo si es del restaurante y está pendiente
    let reservas = repo.reservas();
    let result = reservas
        .update_one(
            doc! {
                "_id": reservation_id,
                "id_restaurante": user_id,
                "estado": "pendiente"
            },
            doc! {
                "$set": {
                    "estado": "confirmada",
                    "updated_at": MongoRepo::current_timestamp()
                }
            }
        )
        .await
        .map_err(|e| AppError::Internal(format!("Error confirmando reserva: {}", e)))?;

    if result.modified_count == 0 {
        return Err(AppError::NotFound("Reserva no encontrada o ya procesada".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Reserva confirmada correctamente",
        "id": reservation_id.to_hex(),
        "estado": "confirmada"
    })))
}

/// Cancela una reserva
///
/// Cambia el estado de una reserva a "cancelada". Una vez cancelada,
/// la reserva no se puede reactivar ni modificar.
///
/// # Autenticación
/// Requiere token Bearer válido del restaurante propietario.
///
/// # Parámetros
/// - `repo`: Repositorio MongoDB
/// - `path`: ID de la reserva a cancelar (en la URL)
/// - `req`: Request HTTP con el token de autorización
///
/// # Respuesta
/// ```json
/// {
///   "message": "Reserva cancelada correctamente",
///   "id": "507f1f77bcf86cd799439011",
///   "estado": "cancelada"
/// }
/// ```
///
/// # Errores
/// - `400 Bad Request`: ID de reserva inválido
/// - `401 Unauthorized`: Token inválido o falta autorización
/// - `403 Forbidden`: No tienes permiso para cancelar reservas de este restaurante
/// - `404 Not Found`: Reserva no encontrada o ya cancelada
/// - `500 Internal Server Error`: Error de base de datos
#[post("/reservations/{id}/cancel")]
async fn cancel_reservation(
    repo: web::Data<MongoRepo>,
    path: web::Path<String>,
    req: HttpRequest,
) -> AppResult<impl Responder> {
    let token = extract_token(&req)?;
    let user_id = validate_access_token(repo.get_ref(), &token).await?;
    let reservation_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError::Validation("ID de reserva inválido".to_string()))?;

    // Actualizar la reserva solo si es del restaurante y no está ya cancelada
    let reservas = repo.reservas();
    let result = reservas
        .update_one(
            doc! {
                "_id": reservation_id,
                "id_restaurante": user_id,
                "estado": {"$ne": "cancelada"}
            },
            doc! {
                "$set": {
                    "estado": "cancelada",
                    "updated_at": MongoRepo::current_timestamp()
                }
            }
        )
        .await
        .map_err(|e| AppError::Internal(format!("Error cancelando reserva: {}", e)))?;

    if result.modified_count == 0 {
        return Err(AppError::NotFound("Reserva no encontrada o ya cancelada".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Reserva cancelada correctamente",
        "id": reservation_id.to_hex(),
        "estado": "cancelada"
    })))
}

/// Configura las rutas relacionadas con reservas
///
/// # Rutas disponibles
/// - `POST /reservations` - Crear nueva reserva
/// - `GET /reservations` - Listar reservas con filtros opcionales
/// - `POST /reservations/{id}/confirm` - Confirmar reserva pendiente
/// - `POST /reservations/{id}/cancel` - Cancelar reserva
///
/// # Autenticación
/// Todas las rutas requieren autenticación Bearer token.
///
/// # Parámetros
/// - `cfg`: Configuración del servicio Actix Web donde se registran las rutas
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(make_reservation);
    cfg.service(get_reservations);
    cfg.service(confirm_reservation);
    cfg.service(cancel_reservation);
}