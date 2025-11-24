use actix::{Context, Handler};
use crate::actores::peer::messages::Eleccion;
use crate::actores::peer::ypf_peer::YpfPeer;

impl Handler<Eleccion> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, _ctx: &mut Context<Self>) -> Self::Result {
        self.cola_envio.as_mut().map(|tx| {
            tx.send(msg.to_bytes()).expect("Fallo al enviar Eleccion a la cola de envio");
        });
    }
}