use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Venta {
    pub id_venta: usize,
    pub id_tarjeta: usize,
    pub id_estacion: usize,
    pub monto: f32,
    pub offline: bool,
    pub estado: EstadoVenta,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum EstadoVenta {
    Pendiente,
    Confirmada,
    Fallida,
}

