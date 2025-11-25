use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tarjeta {
    #[serde(alias = "id_tarjeta")]
    pub id: usize,
    #[serde(alias = "id_empresa")]
    pub empresa_id: usize,
    #[serde(alias = "limite_particular")]
    pub limite_particular: u64,
    pub consumo_actual: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Empresa {
    #[serde(alias = "id_empresa")]
    pub id: usize,
    pub nombre: String,
    pub limite_general: u64,
    pub consumo_actual: f32,
}
