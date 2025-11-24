use actix::{Context, Handler};
use crate::actores::peer::messages::VentaRegistrada;
use crate::actores::peer::ypf_peer::YpfPeer;

impl Handler<VentaRegistrada> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: VentaRegistrada, _ctx: &mut Context<Self>) -> Self::Result {
        let mensaje = if let Ok(m) = msg.to_bytes() {
            m
        } else {
            return
        };
        println!("YpfPeer: encolando venta registrada");
        self.cola_envio.as_mut().map(|tx| {
            tx.send(mensaje).expect("Fallo al enviar VentaRegistrada a la cola de envio");
        });
    }
}