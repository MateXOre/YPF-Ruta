
#[allow(dead_code)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Venta {
    pub id_venta: usize,
    pub id_tarjeta: usize,
    pub id_estacion: usize,
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

impl PartialEq for EstadoVenta {
    fn eq(&self, other: &Self) -> bool {
        *self == *other
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}