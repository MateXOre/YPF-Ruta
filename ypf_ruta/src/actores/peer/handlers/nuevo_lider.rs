use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::NuevoLider;
use actix::{Context, Handler};
use util::{log_debug, log_error};

impl Handler<NuevoLider> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: NuevoLider, _ctx: &mut Context<Self>) -> Self::Result {
        log_debug!(
            self.logger,
            "YpfPeer {}: Recibido nuevo líder: {}",
            self.peer_id,
            msg.id
        );
        self.cola_envio.as_mut().map(|tx| {
            let bytes = msg.to_bytes();
            log_debug!(
                self.logger,
                "YpfPeer {}: Enviando nuevo líder al socket: byte[0] = {}",
                self.peer_id,
                bytes[0]
            );
            if tx.send(bytes).is_err() {
                log_error!(
                    self.logger,
                    "YpfPeer {}: Fallo al enviar NuevoLider a la cola de envio",
                    self.peer_id
                );
            }
        });
    }
}
