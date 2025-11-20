use actix::{Message, Addr};
use std::net::SocketAddr;
use crate::actores::{estacion_cercana::EstacionCercana, surtidor::surtidor::Surtidor};
use util::structs::venta::Venta;

#[derive(Message)]
#[rtype(result = "()")]
pub struct AgregarEstacion {
    pub estacion: Addr<EstacionCercana>,
    pub estacion_id: usize,
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

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct HabilitarSurtidor {
    pub surtidor_id: usize,
    pub surtidor_addr: Addr<Surtidor>,
}


#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InformarVenta {
    pub venta: Venta
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct TransaccionesPorEstacion {
    pub transacciones: Vec<Venta>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ConfirmarTransacciones {
    pub transacciones: Vec<Venta>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct CobrarACliente {
    pub venta: Venta,
    pub surtidor_id: usize,
}

