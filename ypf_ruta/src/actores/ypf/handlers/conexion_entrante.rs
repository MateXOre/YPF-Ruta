use crate::actores::peer::messages::GuardarSocket;
use crate::actores::ypf::messages::ConexionEntrante;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::Handler;

impl Handler<ConexionEntrante> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: ConexionEntrante, ctx: &mut Self::Context) {
        let peer_id = msg.peer_id;
        let socket = msg.socket;

        // Si ya existe, enviar socket; si no, delega en spawn_peer
        if let Some(peer_addr) = self.ypf_peers.get(&peer_id) {
            println!(
                "YpfRuta {}: Peer {} ya existe, enviando socket",
                self.id, peer_id
            );
            peer_addr.do_send(GuardarSocket(socket));
            // NuevoLider se enviará automáticamente cuando el socket esté listo (SocketListo)
        } else {
            println!(
                "YpfRuta {}: Creando nuevo peer {} con socket entrante",
                self.id, peer_id
            );
            self.spawn_peer(peer_id, Some(socket), None, ctx);
        }
    }
}
