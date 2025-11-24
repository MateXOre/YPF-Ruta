use crate::actores::ypf::messages::{IniciarEleccion, PeerDesconectado};
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{AsyncContext, Context, Handler};

impl Handler<PeerDesconectado> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: PeerDesconectado, ctx: &mut Context<Self>) -> Self::Result {
        println!("YpfRuta {}: Peer {} desconectado.", self.id, msg.id);
        self.ypf_peers.remove(&msg.id);

        // Si el líder se desconectó, iniciar elección
        if Some(msg.id) == self.lider {
            self.lider = None;
            println!(
                "YpfRuta {}: El líder {} se ha desconectado. Iniciando elección Bully.",
                self.id, msg.id
            );
            ctx.address().do_send(IniciarEleccion);
        }
    }
}
