// src/db/mod.rs
pub mod models;
pub mod mongodb;

pub use mongodb::{MongoRepo, Restaurant, Mesa, Reserva};

// Re-exports para compatibilidad
pub use MongoRepo as Database;