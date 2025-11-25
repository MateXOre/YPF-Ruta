use crate::actores::peer::messages::Eleccion;
use crate::actores::peer::ypf_peer::YpfPeer;
use actix::{Context, Handler};
use util::log_error;

impl Handler<Eleccion> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(tx) = self.cola_envio.as_mut()
            && tx.send(msg.to_bytes()).is_err()
        {
            log_error!(
                self.logger,
                "YpfPeer {}: Fallo al enviar Eleccion a la cola de envio",
                self.peer_id
            );
        }
    }
}
