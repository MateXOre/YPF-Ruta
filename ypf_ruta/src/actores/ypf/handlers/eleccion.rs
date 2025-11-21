use actix::{Context, Handler};
use crate::actores::peer::messages::{Eleccion};
use crate::actores::ypf::messages::NuevoLider;
use crate::actores::ypf::ypf_actor::YpfRuta;

impl Handler<Eleccion> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, _ctx: &mut Context<Self>) -> Self::Result {
        if self.lider.is_none() {
            self.lider = Some(msg.0);
            println!("YpfRuta {}: Nuevo líder asignado: {}.", self.id, msg.0);
        } else {
            println!("YpfRuta {}: Ya hay un líder actual: {}. No se puede asignar un nuevo líder: {}.", self.id, self.lider.unwrap(), msg.0);
            return;
        }
        
        self.ypf_peers.iter().for_each(|(_, addr)| {
            let response = NuevoLider{ id: msg.0 };
            addr.do_send(response);
        });

        println!("YpfRuta {}: Respuesta enviada a todos los peers: líder asignado {}", self.id, msg.0);
    }
}












