use actix::{Context, Handler};
use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::PersistirEstado;

impl Handler<PersistirEstado> for Gestor {
    type Result = ();

    fn handle(&mut self, _msg: PersistirEstado, _ctx: &mut Context<Self>) -> Self::Result {
        self.persistir_estado_actual();
    }
}