use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::ModificarLimiteGeneral;
use actix::{Context, Handler};

impl Handler<ModificarLimiteGeneral> for Gestor {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: ModificarLimiteGeneral, _ctx: &mut Context<Self>) -> Self::Result {
        self.modificar_limite_general_empresa(msg.id_empresa, msg.nuevo_limite as f32)
    }
}
