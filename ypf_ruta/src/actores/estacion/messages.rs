use std::collections::HashMap;

use crate::actores::estacion::estacion_actor::Estacion;
use actix::{Addr, Message};
use util::structs::venta::Venta;

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
