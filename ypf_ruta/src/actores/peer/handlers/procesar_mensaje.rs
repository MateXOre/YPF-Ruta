use actix::{Context, Handler};
use crate::actores::peer::messages::{Eleccion, ProcesarMensaje, VentaRegistrada};
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::NuevoLider;

impl Handler<ProcesarMensaje> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: ProcesarMensaje, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Byte 0 recibido: {}", msg.bytes[0]);
        match msg.bytes[0] {
            b'0' => {
                // Ping recibido
                self.last_ping = std::time::Instant::now();
                println!("YpfPeer {}: Ping recibido.", self.peer_id);
                if let Some(cola) = self.cola_envio.as_mut() {
                    match cola.send(vec![b'1']) {
                        Ok(_) => println!("YpfPeer {}: Pong enviado exitosamente", self.peer_id),
                        Err(e) => eprintln!("YpfPeer {}: Error enviando pong: {:?}", self.peer_id, e),
                    }
                } else {
                    eprintln!("YpfPeer {}: cola_envio es None, no se puede enviar pong", self.peer_id);
                }
            },
            b'1' => {
                // Pong recibido
                self.last_pong = std::time::Instant::now();
                println!("YpfPeer {}: Pong recibido.", self.peer_id);
            },
            b'3' => {
                let msg = Eleccion::fromBytes(msg.bytes.as_slice());
                self.ypf_local_addr.do_send(msg);
            },
            b'4' => {
                let msg = NuevoLider::fromBytes(msg.bytes.as_slice());
                self.ypf_local_addr.do_send(msg);
            },
            b'5' => {
                let msg = VentaRegistrada::fromBytes(msg.bytes.as_slice());
                //self.ypf_local_addr.do_send(msg);
            },
            _ => {
                eprintln!("YpfPeer {}: Tipo de mensaje desconocido.", self.peer_id);
            }
        }
    }
}