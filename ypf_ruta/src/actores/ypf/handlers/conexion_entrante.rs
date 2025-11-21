use actix::{Actor, ActorFutureExt};
use actix::{AsyncContext, Handler, WrapFuture};
use crate::actores::peer::messages::GuardarSocket;
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::ConexionEntrante;
use crate::actores::ypf::ypf_actor::YpfRuta;

// Handler para conexiones entrantes
impl Handler<ConexionEntrante> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: ConexionEntrante, ctx: &mut Self::Context) {
        let peer_id = msg.peer_id;
        let socket = msg.socket;

        // Verificar si ya existe el peer
        if let Some(peer_addr) = self.ypf_peers.get(&peer_id) {
            println!("YpfRuta {}: Peer {} ya existe, enviando socket", self.id, peer_id);
            peer_addr.do_send(GuardarSocket(socket));
        } else {
            // Crear nuevo peer con el socket entrante
            println!("YpfRuta {}: Creando nuevo peer {} con socket entrante", self.id, peer_id);
            let self_id = self.id;
            let self_addr = ctx.address();

            let fut = async move {
                YpfPeer::new(peer_id, self_id, Some(socket), None, self_addr).await
            };

            let fut = fut.into_actor(self).map(move |peer, act, _ctx| {
                let addr = peer.start();
                act.ypf_peers.insert(peer_id, addr);
                println!("YpfRuta {}: Peer {} registrado desde conexión entrante", act.id, peer_id);
            });

            ctx.spawn(fut);
        }
    }
}