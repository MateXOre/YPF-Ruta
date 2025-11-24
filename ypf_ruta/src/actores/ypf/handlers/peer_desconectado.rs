use crate::actores::ypf::messages::{IniciarEleccion, PeerDesconectado};
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{AsyncContext, Context, Handler};
use util::{log_info, log_warning};

impl Handler<PeerDesconectado> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: PeerDesconectado, ctx: &mut Context<Self>) -> Self::Result {
        log_warning!(
            self.logger,
            "YpfRuta {}: Peer {} desconectado.",
            self.id,
            msg.id
        );
        self.ypf_peers.remove(&msg.id);

        // Si el líder se desconectó, iniciar elección
        if Some(msg.id) == self.lider {
            self.lider = None;
            log_info!(
                self.logger,
                "YpfRuta {}: El líder {} se ha desconectado. Iniciando elección Bully.",
                self.id,
                msg.id
            );
            ctx.address().do_send(IniciarEleccion);
        }
    }
}