//! # API de Restaurantes
//!
//! Este módulo maneja todas las operaciones relacionadas con restaurantes:
//! - Registro de nuevos restaurantes
//! - Login y autenticación
//! - Listado de restaurantes
//! - Validación de tokens de acceso

use actix_web::{post, get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use mongodb::bson::{doc, oid::ObjectId};
use uuid::Uuid;
use super::{AppError, AppResult};
use super::middleware::ErrorLogExt; // ← Añadir este import
use crate::db::{MongoRepo, Restaurant};

/// Estructura para el registro de restaurantes
#[derive(Deserialize)]
struct RegisterRestaurant {
    /// ID del sistema Pispas externo
    objid_pispas: String,
    /// Nombre del restaurante
    name: String,
    /// Contraseña (debería estar hasheada en producción)
    password: String,
    /// Si las reservas se confirman automáticamente
    confirmar_automaticamente: bool,
}

#[derive(Deserialize)]
struct LoginRequest {
    name: String,
    password: String,
}

#[derive(Serialize)]
struct RestaurantInfo {
    id: String,
    nombre: String,
    objid_pispas: String,
    confirmar_automaticamente: bool,
}

// Para debug - incluir contraseñas
#[derive(Serialize)]
struct RestaurantInfoWithPassword {
    id: String,
    nombre: String,
    objid_pispas: String,
    password: String,
    confirmar_automaticamente: bool,
}

/// Registra un nuevo restaurante en el sistema
///
/// # Parámetros
///
/// - `repo`: Referencia al repositorio MongoDB
/// - `data`: Datos del restaurante a registrar
///
/// # Respuesta
///
/// ```json
/// {
///   "access_token": "uuid-token",
///   "message": "Restaurante registrado correctamente",
///   "id": "mongodb-object-id"
/// }
/// ```
///
/// # Errores
///
/// - `400 Bad Request`: Datos de validación incorrectos
/// - `409 Conflict`: El restaurante ya existe
/// - `500 Internal Server Error`: Error de base de datos
#[post("/restaurants/register")]
async fn register_restaurant(
    repo: web::Data<MongoRepo>,
    data: web::Json<RegisterRestaurant>,
) -> AppResult<impl Responder> {
    // Validación básica
    if data.name.is_empty() {
        return Err(AppError::Validation("El nombre del restaurante es requerido".to_string()));
    }

    if data.password.len() < 6 {
        return Err(AppError::Validation("La contraseña debe tener al menos 6 caracteres".to_string()));
    }

    if data.objid_pispas.is_empty() {
        return Err(AppError::Validation("El OBJID de Pispas es requerido".to_string()));
    }

    // Verificar si el restaurante ya existe
    let restaurants = repo.restaurants();

    let existing = restaurants
        .find_one(doc! {
            "$or": [
                {"nombre": &data.name},
                {"objid_pispas": &data.objid_pispas}
            ]
        }) // ← Añadir None como segundo argumento
        .await
        .log_error_context("checking if restaurant exists")
        .map_err(|e| AppError::database("check_restaurant_exists", e))?;

    if existing.is_some() {
        return Err(AppError::Conflict("El restaurante ya existe".to_string()));
    }

    let access_token = Uuid::new_v4().to_string();

    let restaurant = Restaurant {
        id: None,
        objid_pispas: data.objid_pispas.clone(),
        nombre: data.name.clone(),
        password: data.password.clone(),
        confirmar_automaticamente: data.confirmar_automaticamente,
        access_token: access_token.clone(),
        created_at: MongoRepo::current_timestamp(),
    };

    let result = restaurants
        .insert_one(restaurant)
        .await
        .log_error_context("inserting new restaurant")
        .map_err(|e| AppError::database("register_restaurant", e))?;

    Ok(HttpResponse::Ok().json(json!({
        "access_token": access_token,
        "message": "Restaurante registrado correctamente",
        "id": result.inserted_id.as_object_id().unwrap().to_hex()
    })))
}

#[post("/restaurants/login")]
async fn login_restaurant(
    repo: web::Data<MongoRepo>,
    data: web::Json<LoginRequest>,
) -> AppResult<impl Responder> {
    // Validación básica
    if data.name.is_empty() || data.password.is_empty() {
        return Err(AppError::Validation("Nombre y contraseña son requeridos".to_string()));
    }

    let restaurants = repo.restaurants();

    let restaurant = restaurants
        .find_one(doc! {
            "nombre": &data.name,
            "password": &data.password
        })
        .await
        .map_err(|e| AppError::Internal(format!("Error buscando restaurante: {}", e)))?;

    match restaurant {
        Some(restaurant) => {
            Ok(HttpResponse::Ok().json(json!({
                "access_token": restaurant.access_token,
                "id_restaurante": restaurant.id.unwrap().to_hex(),
                "message": "Login exitoso"
            })))
        }
        None => Err(AppError::Unauthorized("Credenciales incorrectas".to_string()))
    }
}

#[get("/restaurants/all")]
async fn list_restaurants(
    repo: web::Data<MongoRepo>,
) -> AppResult<impl Responder> {
    let restaurants = repo.restaurants();

    let cursor = restaurants
        .find(doc! {}) // ← Añadir None como segundo argumento
        .await
        .log_error_context("listing all restaurants")
        .map_err(|e| AppError::database("list_restaurants", e))?;

    let mut results = Vec::new();
    let mut cursor = cursor;

    while cursor.advance().await.map_err(|e| AppError::Internal(format!("Error iterando cursor: {}", e)))? {
        let restaurant = cursor.deserialize_current()
            .map_err(|e| AppError::Internal(format!("Error deserializando restaurant: {}", e)))?;

        results.push(RestaurantInfo {
            id: restaurant.id.unwrap().to_hex(),
            nombre: restaurant.nombre,
            objid_pispas: restaurant.objid_pispas,
            confirmar_automaticamente: restaurant.confirmar_automaticamente,
        });
    }

    Ok(HttpResponse::Ok().json(results))
}

// Endpoint de debug con contraseñas
#[get("/restaurants/all/debug")]
async fn list_restaurants_with_passwords(
    repo: web::Data<MongoRepo>,
) -> AppResult<impl Responder> {
    // ⚠️ ADVERTENCIA: ESTO ES SOLO PARA DEBUG ⚠️
    let restaurants = repo.restaurants();

    let cursor = restaurants
        .find(mongodb::bson::Document::new()) // ← Añadir None como segundo argumento
        .await
        .log_error_context("listing restaurants for debug")
        .map_err(|e| AppError::database("list_restaurants_debug", e))?;

    let mut results = Vec::new();
    let mut cursor = cursor;

    while cursor.advance().await.map_err(|e| AppError::Internal(format!("Error iterando cursor: {}", e)))? {
        let restaurant = cursor.deserialize_current()
            .map_err(|e| AppError::Internal(format!("Error deserializando restaurant: {}", e)))?;

        results.push(RestaurantInfoWithPassword {
            id: restaurant.id.unwrap().to_hex(),
            nombre: restaurant.nombre,
            objid_pispas: restaurant.objid_pispas,
            password: restaurant.password,
            confirmar_automaticamente: restaurant.confirmar_automaticamente,
        });
    }

    Ok(HttpResponse::Ok().json(results))
}

// Nueva función para validar token con MongoDB
pub async fn validate_access_token(
    repo: &MongoRepo,
    token: &str,
) -> AppResult<ObjectId> {
    let restaurants = repo.restaurants();

    let restaurant = restaurants
        .find_one(doc! { "access_token": token }) // ← Añadir None como segundo argumento
        .await
        .log_error_context("validating access token")
        .map_err(|e| AppError::database("validate_token", e))?;

    match restaurant {
        Some(restaurant) => Ok(restaurant.id.unwrap()),
        None => Err(AppError::Unauthorized("Token inválido".to_string()))
    }
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register_restaurant);
    cfg.service(login_restaurant);
    cfg.service(list_restaurants);
    // SOLO para debug local:
    cfg.service(list_restaurants_with_passwords);
}