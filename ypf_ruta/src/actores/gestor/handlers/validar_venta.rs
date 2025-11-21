use actix::{Context, Handler};
use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::{ConsultarEstado, ModificarLimiteGeneral, ModificarLimiteParticular, ValidarVenta};
use crate::actores::gestor::structs::{Empresa, Tarjeta};

impl Handler<ValidarVenta> for Gestor {
    type Result = bool;

    fn handle(&mut self, msg: ValidarVenta, _ctx: &mut Context<Self>) -> Self::Result {
        self.procesar_venta_internal(&msg.0)
    }
}

