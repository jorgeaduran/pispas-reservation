//! # Pispas Reservation Server
//!
//! Servidor web para el sistema de reservas de restaurantes construido con Rust, Actix Web y MongoDB.
//!
//! ## Características principales
//!
//! - **Sistema de restaurantes**: Registro, login y gestión de restaurantes
//! - **Plano de mesas**: Interfaz visual drag-and-drop para organización de mesas
//! - **Sistema de reservas**: Gestión completa del estado de reservas
//! - **API REST**: API completa con autenticación por tokens
//! - **Frontend incluido**: Interfaz web en JavaScript vanilla
//!
//! ## Configuración
//!
//! El servidor se configura mediante variables de entorno (archivo `.env`):
//!
//! ```env
//! # Base de datos MongoDB
//! MONGODB_URI=mongodb://localhost:27017
//! MONGODB_DATABASE=pispas_reservation
//!
//! # Servidor
//! BIND_ADDRESS=0.0.0.0:8080
//!
//! # Logging
//! RUST_LOG=debug,mongodb=info
//! ```
//!
//! ## Ejecución
//!
//! ```bash
//! # 1. Instalar y ejecutar MongoDB
//! # Local: mongod
//! # Docker: docker run -d --name mongo -p 27017:27017 mongo:latest
//!
//! # 2. Configurar variables de entorno
//! cp .env.example .env
//!
//! # 3. Compilar y ejecutar
//! cargo run
//!
//! # 4. Acceder al servidor
//! # http://localhost:8080
//! ```
//!
//! ## Arquitectura
//!
//! ```text
//! Frontend (HTML/CSS/JS)
//!     ↓ HTTP/JSON
//! API REST (Actix Web)
//!     ↓ MongoDB Driver
//! MongoDB Database
//! ```

use actix_files::Files;
use actix_web::{web, App, HttpServer, middleware::Logger};
use std::env;

mod api;
mod db;

/// Función principal que inicia el servidor web
///
/// # Funcionalidad
///
/// 1. Carga variables de entorno desde `.env`
/// 2. Configura el sistema de logging con tracing
/// 3. Establece conexión con MongoDB
/// 4. Crea índices en la base de datos
/// 5. Configura el servidor HTTP con:
///    - Middleware de logging
///    - Rutas de la API
///    - Servicio de archivos estáticos
///    - Redirección de la ruta raíz
/// 6. Inicia el servidor en la dirección especificada
///
/// # Variables de entorno
///
/// - `MONGODB_URI`: URI de conexión a MongoDB (default: mongodb://localhost:27017)
/// - `MONGODB_DATABASE`: Nombre de la base de datos (default: pispas_reservation)
/// - `BIND_ADDRESS`: Dirección y puerto del servidor (default: 0.0.0.0:8080)
/// - `RUST_LOG`: Nivel de logging (default: debug para la app, info para MongoDB)
///
/// # Errores
///
/// Retorna `std::io::Error` si:
/// - No se puede conectar a MongoDB
/// - Error al crear índices en la base de datos
/// - No se puede bindear al puerto especificado
/// - Error general al inicializar el servidor
///
/// # Ejemplos
///
/// ```bash
/// # Ejecutar con configuración por defecto
/// cargo run
///
/// # Ejecutar en puerto diferente
/// BIND_ADDRESS=0.0.0.0:3000 cargo run
///
/// # Ejecutar con MongoDB remoto
/// MONGODB_URI=mongodb://remote:27017 cargo run
/// ```
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    // Configurar sistema de logging con tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("pispas_reservation=debug".parse().unwrap())
                .add_directive("mongodb=info".parse().unwrap())
        )
        .init();

    tracing::info!("Iniciando Pispas Reservation Server con MongoDB... test");

    // Inicializar conexión a MongoDB
    let mongo_repo = match db::MongoRepo::init().await {
        Ok(repo) => {
            tracing::info!("Conexión a MongoDB establecida exitosamente");

            // Intentar crear índices para optimizar consultas
            if let Err(e) = repo.create_indexes().await {
                tracing::warn!("Advertencia creando índices: {}", e);
                // No es un error fatal, continuamos sin índices
            }

            repo
        }
        Err(e) => {
            tracing::error!("Error conectando a MongoDB: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error de MongoDB: {}", e)
            ));
        }
    };

    // Obtener dirección de bind desde variables de entorno
    let bind_address = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    tracing::info!("Servidor iniciando en {}", bind_address);
    tracing::info!("prueba");
    // Crear y configurar el servidor HTTP
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_repo.clone()))
            .wrap(Logger::default())
            .configure(api::init_routes)
            .service(Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(|| async {
                actix_web::HttpResponse::PermanentRedirect()
                    .append_header(("Location", "/static/index.html"))
                    .finish()
            }))
    })
        .bind(&bind_address)?
        .run()
        .await
}