use actix::{Handler, Context};
use crate::actores::estacion::{Estacion, ConexionEstacion};
use crate::actores::estacion::messages::*;
use crate::actores::estacion_cercana::EstacionCercana;


impl Handler<Reenviar> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Reenviar, ctx: &mut Context<Self>) {
        self.enviar_a_siguiente(ctx, msg.0);
    }
}