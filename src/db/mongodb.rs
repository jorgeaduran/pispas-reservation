use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};
use std::env;
use crate::api::AppError;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Restaurant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub objid_pispas: String,
    pub nombre: String,
    pub password: String,
    pub confirmar_automaticamente: bool,
    pub access_token: String,
    pub created_at: i64, // timestamp unix
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mesa {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub id_restaurante: mongodb::bson::oid::ObjectId,
    pub tipo: String,
    pub nombre: String,
    pub pos_x: f32,
    pub pos_y: f32,
    pub size_x: f32,
    pub size_y: f32,
    pub forma: String,
    pub reservable: bool,
    pub min_personas: Option<i32>,
    pub max_personas: Option<i32>,
    pub created_at: i64, // timestamp unix
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reserva {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub id_restaurante: mongodb::bson::oid::ObjectId,
    pub id_mesa: mongodb::bson::oid::ObjectId,
    pub nombre_cliente: String,
    pub email_cliente: String,
    pub telefono_cliente: String,
    pub numero_personas: i32,
    pub fecha: String,
    pub hora: String,
    pub estado: String,
    pub created_at: i64, // timestamp unix
    pub updated_at: i64, // timestamp unix
}

#[derive(Debug, Clone)]
pub struct MongoRepo {
    pub client: Client,
    pub database: Database,
}

impl MongoRepo {
    pub async fn init() -> Result<MongoRepo> {
        let mongo_uri = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        let client = Client::with_uri_str(&mongo_uri)
            .await
            .map_err(|e| AppError::Internal(format!("Error conectando a MongoDB: {}", e)))?;

        let database_name = env::var("MONGODB_DATABASE")
            .unwrap_or_else(|_| "pispas_reservation".to_string());

        let database = client.database(&database_name);

        // Test connection
        database
            .run_command(mongodb::bson::doc! {"ping": 1})
            .await
            .map_err(|e| AppError::Internal(format!("Error validando conexión MongoDB: {}", e)))?;

        tracing::info!("Conexión a MongoDB establecida exitosamente");

        Ok(MongoRepo { client, database })
    }

    pub fn restaurants(&self) -> Collection<Restaurant> {
        self.database.collection("restaurants")
    }

    pub fn mesas(&self) -> Collection<Mesa> {
        self.database.collection("mesas")
    }

    pub fn reservas(&self) -> Collection<Reserva> {
        self.database.collection("reservas")
    }

    // Método para crear índices si es necesario
    pub async fn create_indexes(&self) -> Result<()> {
        use mongodb::{options::IndexOptions, IndexModel};
        use mongodb::bson::doc;

        // Índices para restaurants
        let restaurants = self.restaurants();
        let restaurant_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "objid_pispas": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "nombre": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
            IndexModel::builder()
                .keys(doc! { "access_token": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ];

        restaurants
            .create_indexes(restaurant_indexes)
            .await
            .map_err(|e| AppError::Internal(format!("Error creando índices: {}", e)))?;

        // Índices para mesas
        let mesas = self.mesas();
        let mesa_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "id_restaurante": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "id_restaurante": 1, "nombre": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ];

        mesas
            .create_indexes(mesa_indexes)
            .await
            .map_err(|e| AppError::Internal(format!("Error creando índices mesas: {}", e)))?;

        // Índices para reservas
        let reservas = self.reservas();
        let reservation_indexes = vec![
            IndexModel::builder()
                .keys(doc! { "id_restaurante": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "fecha": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "estado": 1 })
                .build(),
            IndexModel::builder()
                .keys(doc! { "id_mesa": 1, "fecha": 1, "hora": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        ];

        reservas
            .create_indexes(reservation_indexes)
            .await
            .map_err(|e| AppError::Internal(format!("Error creando índices reservas: {}", e)))?;

        tracing::info!("Índices MongoDB creados exitosamente");
        Ok(())
    }

    // Función auxiliar para obtener timestamp actual
    pub fn current_timestamp() -> i64 {
        chrono::Utc::now().timestamp()
    }
}