//! # Manejo de errores con demostración de thiserror
//!
//! Este módulo muestra el poder de thiserror para crear jerarquías de errores rica

use actix_web::{HttpResponse, ResponseError};
use std::fmt;
use std::error::Error; // ← Añadir esta importación
use thiserror::Error;

/// Tipos de error de la aplicación con contexto mejorado
#[derive(Error, Debug)]
pub enum AppError {
    /// Error de base de datos con contexto adicional
    ///
    /// # Ejemplo de uso de thiserror
    /// Este error se genera automáticamente desde mongodb::error::Error
    /// y mantiene la cadena de errores original para mejor debugging.
    #[error("Error de base de datos en operación '{operation}': {source}")]
    Database {
        operation: String,
        #[source] // thiserror automáticamente maneja esto
        source: mongodb::error::Error,
    },

    /// Error de validación con campo específico
    #[error("Error de validación en campo '{field}': {message}")]
    ValidationWithField {
        field: String,
        message: String,
    },

    /// Error de validación general
    #[error("Error de validación: {0}")]
    Validation(String),

    /// Error de autorización con contexto
    #[error("No autorizado para operación '{operation}': {reason}")]
    UnauthorizedWithContext {
        operation: String,
        reason: String,
    },

    /// Error de autorización simple
    #[error("No autorizado: {0}")]
    Unauthorized(String),

    /// Error de recurso no encontrado
    #[error("No encontrado: {resource_type} con ID '{id}'")]
    NotFoundWithId {
        resource_type: String,
        id: String,
    },

    /// Error de no encontrado simple
    #[error("No encontrado: {0}")]
    NotFound(String),

    /// Error de conflicto
    #[error("Conflicto: {0}")]
    Conflict(String),

    /// Error interno con código de rastreo
    #[error("Error interno (trace: {trace_id}): {message}")]
    InternalWithTrace {
        trace_id: String,
        message: String,
    },

    /// Error interno simple
    #[error("Error interno: {0}")]
    Internal(String),
}

// Métodos helper para crear errores con contexto
impl AppError {
    /// Crea un error de base de datos con contexto de operación
    pub fn database(operation: &str, source: mongodb::error::Error) -> Self {
        Self::Database {
            operation: operation.to_string(),
            source,
        }
    }

    /// Crea un error de validación con campo específico
    pub fn validation_field(field: &str, message: &str) -> Self {
        Self::ValidationWithField {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    /// Crea un error de autorización con contexto
    pub fn unauthorized_operation(operation: &str, reason: &str) -> Self {
        Self::UnauthorizedWithContext {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Crea un error de no encontrado con ID
    pub fn not_found_id(resource_type: &str, id: &str) -> Self {
        Self::NotFoundWithId {
            resource_type: resource_type.to_string(),
            id: id.to_string(),
        }
    }

    /// Crea un error interno con trace ID
    pub fn internal_trace(message: &str, trace_id: Option<String>) -> Self {
        Self::InternalWithTrace {
            trace_id: trace_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            message: message.to_string(),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // Log detallado del error antes de responder
        match self {
            Self::Database { operation, source } => {
                tracing::error!(
                    operation = %operation,
                    error = %source,
                    error_chain = ?source.source(),
                    "Database error occurred"
                );
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Error de base de datos".to_string(),
                    message: "Error interno del servidor".to_string(),
                })
            }
            Self::ValidationWithField { field, message } => {
                tracing::warn!(
                    field = %field,
                    message = %message,
                    "Validation error"
                );
                HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Error de validación".to_string(),
                    message: format!("Campo '{}': {}", field, message),
                })
            }
            Self::UnauthorizedWithContext { operation, reason } => {
                tracing::warn!(
                    operation = %operation,
                    reason = %reason,
                    "Unauthorized access attempt"
                );
                HttpResponse::Unauthorized().json(ErrorResponse {
                    error: "No autorizado".to_string(),
                    message: format!("Operación '{}': {}", operation, reason),
                })
            }
            Self::NotFoundWithId { resource_type, id } => {
                tracing::info!(
                    resource_type = %resource_type,
                    id = %id,
                    "Resource not found"
                );
                HttpResponse::NotFound().json(ErrorResponse {
                    error: "No encontrado".to_string(),
                    message: format!("{} con ID '{}' no encontrado", resource_type, id),
                })
            }
            Self::InternalWithTrace { trace_id, message } => {
                tracing::error!(
                    trace_id = %trace_id,
                    message = %message,
                    "Internal error with trace"
                );
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Error interno".to_string(),
                    message: format!("Error interno (trace: {})", trace_id),
                })
            }
            // Fallback para otros errores
            error => {
                tracing::error!(
                    error = %error,
                    error_chain = ?error.source(),
                    "General error"
                );
                HttpResponse::InternalServerError().json(ErrorResponse {
                    error: "Error".to_string(),
                    message: error.to_string(),
                })
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub type AppResult<T> = Result<T, AppError>;

// Conversión automática desde mongodb::error::Error
impl From<mongodb::error::Error> for AppError {
    fn from(error: mongodb::error::Error) -> Self {
        Self::Database {
            operation: "database_operation".to_string(),
            source: error,
        }
    }
}


// Conversión desde errores de ObjectId
impl From<mongodb::bson::oid::Error> for AppError {
    fn from(e: mongodb::bson::oid::Error) -> Self {
        Self::validation_field("ObjectId", &e.to_string())
    }
}

pub trait ResultExt<T> {
    fn map_err_validation(self, message: &str) -> AppResult<T>;
    fn map_err_internal(self, message: &str) -> AppResult<T>;
    fn map_err_db_operation(self, operation: &str) -> AppResult<T>;
}
impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::error::Error + Send + 'static,
{
    fn map_err_validation(self, message: &str) -> AppResult<T> {
        self.map_err(|e| AppError::Validation(format!("{}: {}", message, e)))
    }

    fn map_err_internal(self, message: &str) -> AppResult<T> {
        self.map_err(|e| AppError::internal_trace(&format!("{}: {}", message, e), None))
    }

    fn map_err_db_operation(self, operation: &str) -> AppResult<T> {
        // versión simplificada y segura
        self.map_err(|e| AppError::internal_trace(&format!("{}: {}", operation, e), None))
    }
}
