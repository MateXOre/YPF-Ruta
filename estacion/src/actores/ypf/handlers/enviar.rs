use crate::actores::ypf::messages::EnviarYpf;
use crate::actores::ypf::ypf_ruta::Ypf;
use actix::prelude::*;

impl Handler<EnviarYpf> for Ypf {
    type Result = ();

    fn handle(&mut self, msg: EnviarYpf, _ctx: &mut Context<Self>) {
        let _ = self.enviar_por_socket(msg.bytes);
    }
}
