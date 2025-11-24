use crate::actores::ypf::messages::{NuevoLider, SocketListo};
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Context, Handler};
use util::log_debug;

impl Handler<SocketListo> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: SocketListo, _ctx: &mut Context<Self>) -> Self::Result {
        let peer_id = msg.peer_id;
        log_debug!(
            self.logger,
            "YpfRuta {}: Socket del peer {} está listo",
            self.id,
            peer_id
        );

        // Si somos líderes, enviar información de líder ahora que el socket está listo
        if let Some(lider_id) = self.lider
            && lider_id == self.id
        {
            if let Some(peer_addr) = self.ypf_peers.get(&peer_id) {
                log_debug!(
                    self.logger,
                    "YpfRuta {}: Enviando NuevoLider al peer {} (socket listo)",
                    self.id,
                    peer_id
                );
                peer_addr.do_send(NuevoLider { id: lider_id });
            }
        }
    }
}
