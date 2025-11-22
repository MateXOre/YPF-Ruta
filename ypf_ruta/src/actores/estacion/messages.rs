use actix::{Addr, Message};
use crate::actores::estacion::estacion_actor::Estacion;
use crate::actores::gestor::structs::Venta;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ValidarVentas {
    pub ventas: Vec<Venta>,
    pub from: Addr<Estacion>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ResultadoVentas {
    pub ventas: Vec<Venta>,
}