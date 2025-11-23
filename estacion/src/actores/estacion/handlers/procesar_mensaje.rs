use actix::prelude::*;
use crate::actores::estacion::messages::{deserialize_message, MessageType};
use crate::actores::estacion::{Estacion, ProcesarMensaje};

impl Handler<ProcesarMensaje> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ProcesarMensaje, ctx: &mut Self::Context) -> Self::Result {
        match deserialize_message(&msg.bytes) {
            Ok(message) => match message {
                MessageType::Reenviar(m) => ctx.address().do_send(m),
                MessageType::Eleccion(m) => ctx.address().do_send(m),
                MessageType::NotificarLider(m) => ctx.address().do_send(m),
                MessageType::InformarVenta(m) => ctx.address().do_send(m),
                MessageType::ConfirmarTransacciones(m) => ctx.address().do_send(m),
                MessageType::IdentificarEstacion(m) => ctx.address().do_send(m),
                MessageType::ResultadoVentas(m) => ctx.address().do_send(m),
                MessageType::InformarVentasOffline(m) => ctx.address().do_send(m),
                MessageType::TransaccionesPorEstacion(m) => ctx.address().do_send(m),
            }
            Err(e) => eprintln!("(Procesar) Error deserializando: {}", e),
        }
    }
}
