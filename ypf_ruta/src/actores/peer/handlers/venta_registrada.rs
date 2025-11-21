use actix::{Context, Handler};
use crate::actores::peer::messages::VentaRegistrada;
use crate::actores::peer::ypf_peer::YpfPeer;

impl Handler<VentaRegistrada> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: VentaRegistrada, _ctx: &mut Context<Self>) -> Self::Result {
        self.cola_envio.as_mut().map(|tx| {
            tx.send(msg.to_bytes()).expect("Fallo al enviar VentaRegistrada a la cola de envio");
        });
    }
}