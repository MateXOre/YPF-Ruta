use actix::{Context, Handler};
use crate::actores::peer::messages::Eleccion;
use crate::actores::ypf::messages::{NuevoLider, PeerDesconectado};
use crate::actores::ypf::ypf_actor::YpfRuta;

impl Handler<NuevoLider> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: NuevoLider, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(lider_actual) = self.lider {
            println!("Ya hay un líder actual: {}. No se puede asignar un nuevo líder: {}.", lider_actual, msg.id);
        } else {
            self.lider = Some(self.id);
            println!("Nuevo líder asignado: {} (yo).", self.lider.unwrap_or(0));
        }
    }
}