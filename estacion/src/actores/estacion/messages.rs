use actix::{Message, Addr};
use std::net::SocketAddr;

use crate::actores::estacion_cercana::EstacionCercana;

#[derive(Message)]
#[rtype(result = "()")]
pub struct AgregarEstacion {
    pub peer: Addr<EstacionCercana>,
    pub peer_addr: SocketAddr,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Reenviar(pub String);

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Eleccion {
    pub aspirantes_ids: Vec<usize>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct NotificarLider {
    pub id_lider: usize,
    pub id_iniciador: usize,
}