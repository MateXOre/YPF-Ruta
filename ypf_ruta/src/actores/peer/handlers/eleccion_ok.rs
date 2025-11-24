use crate::actores::peer::messages::EleccionOk;
use crate::actores::peer::ypf_peer::YpfPeer;
use actix::{Context, Handler};

impl Handler<EleccionOk> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: EleccionOk, _ctx: &mut Context<Self>) -> Self::Result {
        self.cola_envio.as_mut().map(|tx| {
            tx.send(msg.to_bytes())
                .expect("Fallo al enviar EleccionOk a la cola de envio");
        });
    }
}
