use actix::{Context, Handler};
use crate::actores::ypf::messages::AgregarEmpresa;
use crate::actores::ypf::YpfRuta;

impl Handler<AgregarEmpresa> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: AgregarEmpresa, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[YpfRuta {}] Agregando empresa {} a empresas activas", self.id, msg.empresa_id);
        self.empresas_activas.insert(msg.empresa_id, msg.empresa);
    }
}

