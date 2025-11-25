use crate::actores::ypf::YpfRuta;
use crate::actores::ypf::empresa_conectada::Enviar;
use crate::actores::ypf::messages::EnviarBytesEmpresa;
use actix::prelude::*;

impl Handler<EnviarBytesEmpresa> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: EnviarBytesEmpresa, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(empresa) = self.empresas_activas.get(&msg.empresa_id) {
            empresa.do_send(Enviar { bytes: msg.bytes });
        }
    }
}
