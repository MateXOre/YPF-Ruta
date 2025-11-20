use actix::{Handler, Context};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;


impl Handler<Reenviar> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Reenviar, ctx: &mut Context<Self>) {
        println!("Recibimos un mensaje Reenviar");
        self.enviar_a_siguiente(ctx, msg.0);
    }
}