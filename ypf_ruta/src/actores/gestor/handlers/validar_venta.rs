use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::ValidarVenta;
use actix::{Context, Handler, Response};
use util::structs::venta::EstadoVenta;

impl Handler<ValidarVenta> for Gestor {
    type Result = Response<EstadoVenta>;

    fn handle(&mut self, msg: ValidarVenta, _ctx: &mut Context<Self>) -> Self::Result {
        Response::reply(self.procesar_venta_internal(&msg.0))
    }
}
