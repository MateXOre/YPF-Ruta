use actix::{Context, Handler};
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::NuevoLider;

impl Handler<NuevoLider> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: NuevoLider, _ctx: &mut Context<Self>) -> Self::Result {
        self.cola_envio.as_mut().map(|tx| {
            tx.send(msg.toBytes()).expect("Fallo al enviar VentaRegistrada a la cola de envio");
        });
    }
}