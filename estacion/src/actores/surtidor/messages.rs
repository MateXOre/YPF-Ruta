use actix::Message;
use util::structs::venta::Venta;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InformarVenta {
    pub(crate) venta: Venta,
    pub(crate) surtidor_id: usize,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ResultadoVenta {
    pub exito: bool,
    pub id_venta: i32,
}