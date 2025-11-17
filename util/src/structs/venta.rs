
#[allow(dead_code)]
#[derive(Clone)]
pub struct Venta {
    pub id_venta: i32,
    pub id_tarjeta: i32,
    pub id_estacion: i32,
    pub monto: f32,
    pub offline: bool,
    pub estado: EstadoVenta,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum EstadoVenta {
    Pendiente,
    Confirmada,
    Fallida,
}