use actix::{Addr, Message};
use crate::actores::ypf_ruta::YpfRuta;

// Mensaje para procesar entrada de consola
#[derive(Message)]
#[rtype(result = "()")]
pub struct ResponderConsola {
    pub linea: String,
}

// Mensaje para procesar datos recibidos del socket
#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcesarMensajeSocket {
    pub bytes: Vec<u8>,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct ConectadoAypfRuta {
    pub addr: Addr<YpfRuta>,
}