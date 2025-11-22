use actix::{Context, Handler};
use crate::actores::estacion::{Estacion, IdentificarEstacion};

impl Handler<IdentificarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: IdentificarEstacion, _ctx: &mut Context<Self>) {
        println!("[{}] Este mensaje no debería existir, enviado desde: {}", self.id, msg.id);
    }
}