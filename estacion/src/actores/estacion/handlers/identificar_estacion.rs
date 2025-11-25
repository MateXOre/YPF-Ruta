use crate::actores::estacion::{Estacion, IdentificarEstacion};
use actix::{Context, Handler};

impl Handler<IdentificarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: IdentificarEstacion, _ctx: &mut Context<Self>) {
        println!(
            "[{}] Este mensaje no debería existir, enviado desde: {}",
            self.id, msg.id
        );
    }
}
