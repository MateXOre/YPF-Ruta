use actix::{AsyncContext, Context, Handler};
use crate::actores::peer::messages::{Eleccion, EleccionOk};
use crate::actores::ypf::messages::{NuevoLider, IniciarEleccion, EleccionTimeout};
use crate::actores::ypf::ypf_actor::YpfRuta;
use std::time::Duration;

// Handler para iniciar elección (Bully Algorithm)
impl Handler<IniciarEleccion> for YpfRuta {
    type Result = ();

    fn handle(&mut self, _msg: IniciarEleccion, ctx: &mut Context<Self>) -> Self::Result {
        if self.en_eleccion {
            println!("YpfRuta {}: Ya hay una elección en curso", self.id);
            return;
        }

        println!("YpfRuta {}: Iniciando elección Bully", self.id);
        self.en_eleccion = true;
        self.respuestas_recibidas = 0;

        // Enviar ELECTION a todos los nodos con ID mayor
        let mut envio_a_mayores = false;
        for (peer_id, peer_addr) in self.ypf_peers.iter() {
            if *peer_id > self.id {
                println!("YpfRuta {}: Enviando ELECTION al peer {}", self.id, peer_id);
                peer_addr.do_send(Eleccion(self.id));
                envio_a_mayores = true;
            }
        }

        if !envio_a_mayores {
            // No hay nodos con ID mayor, me declaro líder
            println!("YpfRuta {}: No hay nodos con ID mayor. Me declaro líder.", self.id);
            self.declarar_lider(ctx);
        } else {
            // Esperar timeout para respuestas (2 segundos)
            ctx.run_later(Duration::from_secs(2), |act, ctx| {
                ctx.address().do_send(EleccionTimeout);
            });
        }
    }
}

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

// Handler cuando recibo respuesta OK de otro nodo
impl Handler<EleccionOk> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: EleccionOk, _ctx: &mut Context<Self>) -> Self::Result {
        let responder_id = msg.0;
        println!("YpfRuta {}: Recibido OK del nodo {}. Cancelo mi candidatura.", self.id, responder_id);
        self.respuestas_recibidas += 1;
        // No me declaro líder, espero que el nodo con mayor ID lo haga
    }
}

// Handler para timeout de elección
impl Handler<EleccionTimeout> for YpfRuta {
    type Result = ();

    fn handle(&mut self, _msg: EleccionTimeout, ctx: &mut Context<Self>) -> Self::Result {
        if !self.en_eleccion {
            return;
        }

        if self.respuestas_recibidas == 0 {
            // No recibí respuestas, me declaro líder
            println!("YpfRuta {}: Timeout alcanzado sin respuestas. Me declaro líder.", self.id);
            self.declarar_lider(ctx);
        } else {
            println!("YpfRuta {}: Recibí {} respuestas OK. Esperando anuncio del nuevo líder.", self.id, self.respuestas_recibidas);
            self.en_eleccion = false;
        }
    }
}

impl YpfRuta {
    fn declarar_lider(&mut self, _ctx: &mut Context<Self>) {
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












