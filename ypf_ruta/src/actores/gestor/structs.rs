use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct Venta {
//     #[serde(alias = "id_venta")]
//     pub id: u64,
//     #[serde(alias = "id_tarjeta")]
//     pub id_tarjeta: u64,
//     #[serde(alias = "id_estacion")]
//     pub id_estacion: u64,
//     pub monto: u64,
//     #[serde(default)]
//     pub fecha: String,
// }

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
