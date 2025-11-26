use crate::actores::estacion::messages::*;
use crate::actores::estacion::Estacion;
use actix::{Context, Handler};
use util::log_info;

impl Handler<Reenviar> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Reenviar, ctx: &mut Context<Self>) {
        log_info!(
            self.logger,
            "[{}] Recibimos un mensaje Reenviar a la siguiente estación {}",
            self.id,
            self.siguiente_estacion
        );

        self.enviar_a_siguiente(ctx, msg.bytes);

        let siguiente_correcto = if self.id + 1 >= self.todas_las_estaciones.len() {
            0
        } else {
            self.id + 1
        };

        self.siguiente_estacion = siguiente_correcto;

        log_info!(self.logger, "[{}] Actualizamos siguiente estación {}", self.id, self.siguiente_estacion);
    }
}
