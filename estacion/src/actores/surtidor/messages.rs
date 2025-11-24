use actix::Message;
use util::structs::venta::Venta;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ResultadoVenta {
    pub exito: bool,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Detenerme;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct CargarCombustible {
    pub venta: Venta,
}
