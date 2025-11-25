use actix::{Context, Handler};
use crate::actores::empresa::Empresa;
use crate::actores::empresa::messages::ProcesarMensajeSocket;

impl Handler<ProcesarMensajeSocket> for Empresa {
    type Result = ();
    fn handle(&mut self, msg: ProcesarMensajeSocket, _: &mut Context<Self>) {
        println!("[Empresa {}] Mensaje recibido del socket: {} bytes", self.id, msg.bytes.len());
        // Aquí puedes procesar los bytes según tus necesidades
    }
}