use crate::actores::peer::messages::VentaRegistrada;
use crate::actores::peer::ypf_peer::YpfPeer;
use actix::{Context, Handler};
use util::{log_debug, log_error};

impl Handler<VentaRegistrada> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: VentaRegistrada, _ctx: &mut Context<Self>) -> Self::Result {
        let mensaje = if let Ok(m) = msg.to_bytes() {
            m
        } else {
            return;
        };
        log_debug!(self.logger, "YpfPeer: encolando venta registrada");
        if let Some(tx) = self.cola_envio.as_mut()
            && tx.send(mensaje).is_err()
        {
            log_error!(
                self.logger,
                "YpfPeer {}: Fallo al enviar VentaRegistrada a la cola de envio",
                self.peer_id
            );
        }
    }
}
