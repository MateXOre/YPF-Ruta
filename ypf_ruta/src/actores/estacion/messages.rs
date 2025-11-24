use std::collections::HashMap;

use actix::{Addr, Message};
use util::structs::venta::Venta;
use crate::actores::estacion::estacion_actor::Estacion;

pub type Solicitud = HashMap<usize, HashMap<usize, Vec<Venta>>>;
pub type Resultado = HashMap<usize, HashMap<usize, Vec<(usize, bool)>>>;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ValidarVentas {
    pub ventas: Solicitud,
    pub from: Addr<Estacion>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ResultadoVentas {
    pub ventas: Resultado,
}