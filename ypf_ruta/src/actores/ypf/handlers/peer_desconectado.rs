use actix::{Context, Handler};
use crate::actores::peer::messages::Eleccion;
use crate::actores::ypf::messages::PeerDesconectado;
use crate::actores::ypf::ypf_actor::YpfRuta;

impl Handler<PeerDesconectado> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: PeerDesconectado, _ctx: &mut Context<Self>) -> Self::Result {
        self.ypf_peers.remove(&msg.id);
        if msg.id == self.lider.unwrap_or(usize::MAX) {
            self.lider = None;
            println!("YpfRuta {}: El líder se ha desconectado.", self.id);
            
            for (_, addr) in self.ypf_peers.iter() {
                addr.do_send(Eleccion(self.id));
            }
        }
        println!("YpfRuta {}: Peer {} desconectado y removido.", self.id, msg.id);
    }
}