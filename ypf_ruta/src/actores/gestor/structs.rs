use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Venta {
    #[serde(alias = "id_venta")]
    pub id: u64,
    #[serde(alias = "id_tarjeta")]
    pub tarjeta_id: u64,
    #[serde(alias = "id_estacion")]
    pub estacion_id: u64,
    pub monto: u64,
    #[serde(default)]
    pub fecha: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tarjeta {
    #[serde(alias = "id_tarjeta")]
    pub id: u64,
    #[serde(alias = "id_empresa")]
    pub empresa_id: u64,
    #[serde(alias = "limite_particular")]
    pub limite_particular: u64,
    pub consumo_actual: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Empresa {
    #[serde(alias = "id_empresa")]
    pub id: u64,
    pub nombre: String,
    pub limite_general: u64,
    pub consumo_actual: u64,
}
