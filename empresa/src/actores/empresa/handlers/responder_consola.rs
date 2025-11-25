use actix::{Context, Handler};
use crate::actores::empresa::Empresa;
use crate::actores::empresa::messages::ResponderConsola;

impl Handler<ResponderConsola> for Empresa {
    type Result = ();
    fn handle(&mut self, msg: ResponderConsola, _: &mut Context<Self>) {
        println!("[Empresa {}] Entrada recibida: {}", self.id, msg.linea);
    }
}