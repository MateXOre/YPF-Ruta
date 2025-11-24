use crate::actores::peer::messages::EleccionOk;
use crate::actores::peer::ypf_peer::YpfPeer;
use actix::{Context, Handler};
use util::log_error;

impl Handler<EleccionOk> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: EleccionOk, _ctx: &mut Context<Self>) -> Self::Result {
        self.cola_envio.as_mut().map(|tx| {
            if tx.send(msg.to_bytes()).is_err() {
                log_error!(
                    self.logger,
                    "YpfPeer {}: Fallo al enviar EleccionOk a la cola de envio",
                    self.peer_id
                );
            }
        });
    }
}
