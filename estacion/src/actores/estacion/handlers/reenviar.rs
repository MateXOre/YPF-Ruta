use actix::{Handler, Context};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;




impl Handler<Reenviar> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Reenviar, ctx: &mut Context<Self>) {
        println!("Recibimos un mensaje Reenviar a la siguiente estacion {}", self.siguiente_estacion);

        // 1. reenviar
        self.enviar_a_siguiente(ctx, msg.bytes);

        // 2. recalcular el siguiente
        let siguiente_correcto = if self.id + 1 >= self.todas_las_estaciones.len() {
            0
        } else {
            self.id + 1
        };

        self.siguiente_estacion = siguiente_correcto;

        println!("Actualizamos siguiente estación {}", self.siguiente_estacion);
    }
}