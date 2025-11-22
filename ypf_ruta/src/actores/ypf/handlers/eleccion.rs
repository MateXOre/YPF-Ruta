use actix::{AsyncContext, Context, Handler};
use crate::actores::peer::messages::{Eleccion, EleccionOk};
use crate::actores::ypf::messages::{NuevoLider, IniciarEleccion, EleccionTimeout};
use crate::actores::ypf::ypf_actor::YpfRuta;

// Handler cuando recibo un mensaje ELECTION de otro nodo
impl Handler<Eleccion> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, ctx: &mut Context<Self>) -> Self::Result {
        let sender_id = msg.0;
        println!("YpfRuta {}: Recibido ELECTION del nodo {}", self.id, sender_id);

        if sender_id < self.id {
            // Responder OK al nodo con ID menor
            println!("YpfRuta {}: Enviando OK al nodo {} e iniciando mi propia elección", self.id, sender_id);
            
            if let Some(peer_addr) = self.ypf_peers.get(&sender_id) {
                peer_addr.do_send(EleccionOk(self.id));
            }

            // Iniciar mi propia elección si no estoy en una
            if !self.en_eleccion {
                ctx.address().do_send(IniciarEleccion);
            }
        } else {
            println!("YpfRuta {}: Recibido ELECTION de nodo con ID mayor o igual ({}), ignorando", self.id, sender_id);
        }
    }
}

impl YpfRuta {
    pub(crate) fn declarar_lider(&mut self, _ctx: &mut Context<Self>) {
        self.lider = Some(self.id);
        self.en_eleccion = false;
        println!("🎖️  YpfRuta {}: Soy el nuevo LÍDER", self.id);

        // Enviar COORDINATOR a todos los nodos
        for (peer_id, peer_addr) in self.ypf_peers.iter() {
            println!("YpfRuta {}: Enviando COORDINATOR al peer {}", self.id, peer_id);
            peer_addr.do_send(NuevoLider { id: self.id });
        }
    }
}












