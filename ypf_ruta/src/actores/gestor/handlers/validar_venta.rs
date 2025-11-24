use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::ValidarVenta;
use actix::{Context, Handler};

impl Handler<ValidarVenta> for Gestor {
    type Result = bool;

    fn handle(&mut self, msg: ValidarVenta, _ctx: &mut Context<Self>) -> Self::Result {
        self.procesar_venta_internal(&msg.0)
    }
}
