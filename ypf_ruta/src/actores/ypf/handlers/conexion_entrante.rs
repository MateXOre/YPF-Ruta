use actix::{Actor, ActorFutureExt};
use actix::{AsyncContext, Handler, WrapFuture};
use crate::actores::peer::messages::GuardarSocket;
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::{ConexionEntrante, NuevoLider};
use crate::actores::ypf::ypf_actor::YpfRuta;

impl Handler<ConexionEntrante> for YpfRuta {
    type Result = ();
    
    fn handle(&mut self, msg: ConexionEntrante, ctx: &mut Self::Context) {
        let peer_id = msg.peer_id;
        let socket = msg.socket;

        // Si ya existe, enviar socket; si no, delega en spawn_peer
        if let Some(peer_addr) = self.ypf_peers.get(&peer_id) {
            println!("YpfRuta {}: Peer {} ya existe, enviando socket", self.id, peer_id);
            peer_addr.do_send(GuardarSocket(socket));
        } else {
            println!("YpfRuta {}: Creando nuevo peer {} con socket entrante", self.id, peer_id);
            self.spawn_peer(peer_id, Some(socket), None, ctx);
        }
    }
}