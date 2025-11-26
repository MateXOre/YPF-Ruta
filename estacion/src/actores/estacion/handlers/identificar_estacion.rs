use crate::actores::estacion::{Estacion, IdentificarEstacion};
use actix::{Context, Handler};
use util::log_warning;

impl Handler<IdentificarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: IdentificarEstacion, _ctx: &mut Context<Self>) {
        log_warning!(self.logger, "[{}] Este mensaje no debería existir, enviado desde: {}", self.id, msg.id);
    }
}
