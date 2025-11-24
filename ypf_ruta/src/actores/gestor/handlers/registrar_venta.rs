use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::RegistrarVenta;
use actix::{Context, Handler};

impl Handler<RegistrarVenta> for Gestor {
    type Result = ();

    fn handle(&mut self, msg: RegistrarVenta, _ctx: &mut Context<Self>) -> Self::Result {
        self.crear_venta(msg.0)
    }
}
