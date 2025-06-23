//! # Módulo API
//!
//! Este módulo contiene todas las rutas y controladores de la API REST.
//!
//! ## Módulos principales
//!
//! - [`restaurant`] - Gestión de restaurantes (registro, login, listado)
//! - [`table`] - Gestión de mesas (crear, listar, eliminar)
//! - [`reservation`] - Gestión de reservas (crear, confirmar, cancelar)
//! - [`visual`] - Endpoints para el plano visual
//! - [`errors`] - Manejo de errores de la aplicación

pub mod restaurant;
pub mod reservation;
pub mod table;
pub mod visual;
pub mod errors;
mod middleware;

// Re-exportar tipos comunes para facilitar su uso
pub use errors::{AppError, AppResult, ErrorResponse, ResultExt};

use actix_web::web;

/// Configura todas las rutas de la API
///
/// Esta función centraliza la configuración de todas las rutas disponibles:
///
/// ## Rutas configuradas
///
/// - `/restaurants/*` - Ver [`restaurant::routes`]
/// - `/tables/*` - Ver [`table::routes`]
/// - `/reservations/*` - Ver [`reservation::routes`]
/// - `/visual/*` - Ver [`visual::routes`]
///
/// # Parámetros
///
/// - `cfg`: Configuración del servicio Actix Web donde se registran las rutas
///
/// # Ejemplo
///
/// ```no_run
/// use actix_web::{web, App};
/// use pispas_reservation::api;
///
/// let app = App::new()
///     .configure(api::init_routes);
/// ```
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    reservation::routes(cfg);
    restaurant::routes(cfg);
    table::routes(cfg);
    visual::routes(cfg);
}