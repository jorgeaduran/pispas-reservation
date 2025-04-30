use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Restaurante {
    pub id: i32,
    pub objid_pispas: String,
    pub nombre: String,
    pub password: String,
    pub confirmar_automaticamente: bool,
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reserva {
    pub id: i32,
    pub id_restaurante: i32,
    pub id_mesa: i32,
    pub nombre_cliente: String,
    pub email_cliente: String,
    pub telefono_cliente: String,
    pub numero_personas: i32,
    pub fecha: String,
    pub hora: String,
    pub estado: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanoElemento {
    pub id: i32,
    pub id_restaurante: i32,
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
}
