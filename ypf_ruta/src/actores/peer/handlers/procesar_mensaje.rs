use crate::actores::peer::messages::{Eleccion, EleccionOk, ProcesarMensaje, VentaRegistrada};
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::NuevoLider;
use actix::{Context, Handler};
use util::{log_debug, log_error};

impl Handler<ProcesarMensaje> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: ProcesarMensaje, _ctx: &mut Context<Self>) -> Self::Result {
        match msg.bytes[0] {
            b'0' => {
                // Ping recibido
                self.last_ping = std::time::Instant::now();
                log_debug!(self.logger, "YpfPeer {}: Ping recibido.", self.peer_id);
                if let Some(cola) = self.cola_envio.as_mut() {
                    match cola.send(vec![b'1', b'\n']) {
                        Ok(_) => log_debug!(
                            self.logger,
                            "YpfPeer {}: Pong enviado exitosamente",
                            self.peer_id
                        ),
                        Err(e) => {
                            log_error!(
                                self.logger,
                                "YpfPeer {}: Error enviando pong: {:?}",
                                self.peer_id,
                                e
                            );
                        }
                    }
                } else {
                    log_error!(
                        self.logger,
                        "YpfPeer {}: cola_envio es None, no se puede enviar pong",
                        self.peer_id
                    );
                }
            }
            b'1' => {
                // Pong recibido
                self.last_pong = std::time::Instant::now();
                log_debug!(self.logger, "YpfPeer {}: Pong recibido.", self.peer_id);
            }
            b'3' => {
                // Mensaje ELECTION
                let msg = Eleccion::from_bytes(msg.bytes.as_slice());
                self.ypf_local_addr.do_send(msg);
            }
            b'4' => {
                // Mensaje COORDINATOR (NuevoLider)
                let msg = NuevoLider::from_bytes(msg.bytes.as_slice());
                self.ypf_local_addr.do_send(msg);
            }
            b'6' => {
                // Mensaje OK (respuesta a ELECTION)
                let msg = EleccionOk::from_bytes(msg.bytes.as_slice());
                self.ypf_local_addr.do_send(msg);
            }
            b'5' => {
                let msg = VentaRegistrada::from_bytes(msg.bytes.as_slice());
                self.ypf_local_addr.do_send(msg);
            }
            _ => {
                log_error!(
                    self.logger,
                    "YpfPeer {}: Tipo de mensaje desconocido: {}",
                    self.peer_id,
                    msg.bytes[0]
                );
            }
        }
    }
}
