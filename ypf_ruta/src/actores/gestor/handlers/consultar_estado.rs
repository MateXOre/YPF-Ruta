use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::ConsultarEstado;
use crate::actores::gestor::structs::{Empresa, Tarjeta};
use actix::{Context, Handler};

impl Handler<ConsultarEstado> for Gestor {
    type Result = Option<(Empresa, Vec<Tarjeta>)>;

    fn handle(&mut self, msg: ConsultarEstado, _ctx: &mut Context<Self>) -> Self::Result {
        self.consultar_estado_empresa_internal(msg.0)
    }
}
