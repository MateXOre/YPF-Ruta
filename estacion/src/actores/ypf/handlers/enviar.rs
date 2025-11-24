use actix::prelude::*;
use crate::actores::ypf::ypf_ruta::Ypf;
use crate::actores::ypf::messages::EnviarYpf;

impl Handler<EnviarYpf> for Ypf {
    type Result = ();

    fn handle(&mut self, msg: EnviarYpf, _ctx: &mut Context<Self>) {
        let _ = self.enviar_por_socket(msg.bytes);
    }
}