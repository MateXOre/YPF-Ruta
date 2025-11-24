use actix::{Context, Handler};
use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::ModificarLimiteParticular;

impl Handler<ModificarLimiteParticular> for Gestor {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: ModificarLimiteParticular, _ctx: &mut Context<Self>) -> Self::Result {
        self.modificar_limite_particular_tarjeta(msg.id_tarjeta, msg.nuevo_limite as f32)
    }
}