use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Venta {
    pub id_venta: usize,
    pub id_tarjeta: usize,
    pub id_estacion: usize,
    pub monto: f32,
    pub offline: bool,
    pub estado: EstadoVenta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EstadoVenta {
    Pendiente,
    Confirmada,
    Rechazada,
}
