use crate::actores::ypf::messages::ProcesarMensajeEmpresa;
use crate::actores::ypf::YpfRuta;
use crate::actores::ypf::messages::{deserialize_message, MessageType};
use actix::prelude::*;

impl Handler<ProcesarMensajeEmpresa> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: ProcesarMensajeEmpresa, ctx: &mut Self::Context) -> Self::Result {
        match deserialize_message(&msg.bytes) {
            Ok(message) => match message {
                MessageType::ConfigurarLimite(m) => ctx.address().do_send(m),
                MessageType::ConfigurarLimiteGeneral(m) => ctx.address().do_send(m),
                MessageType::GastosEmpresa(m) => ctx.address().do_send(m),
            },
            Err(e) => eprintln!("(Procesar) Error deserializando: {}", e),
        }
    }
}