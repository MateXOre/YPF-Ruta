use crate::actores::peer::messages::Eleccion;
use crate::actores::ypf::messages::{EleccionTimeout, IniciarEleccion};
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{AsyncContext, Context, Handler};
use util::{log_debug, log_info, log_warning};
use std::time::Duration;

impl Handler<IniciarEleccion> for YpfRuta {
    type Result = ();

    fn handle(&mut self, _msg: IniciarEleccion, ctx: &mut Context<Self>) -> Self::Result {
        if self.en_eleccion {
            log_warning!(
                self.logger,
                "YpfRuta {}: Ya hay una elección en curso",
                self.id
            );
            return;
        }

        log_info!(
            self.logger,
            "YpfRuta {}: Iniciando elección Bully",
            self.id
        );
        self.en_eleccion = true;
        self.respuestas_recibidas = 0;

        // Enviar ELECTION a todos los nodos con ID mayor
        let mut envio_a_mayores = false;
        for (peer_id, peer_addr) in self.ypf_peers.iter() {
            if *peer_id > self.id {
                log_debug!(
                    self.logger,
                    "YpfRuta {}: Enviando ELECCION al peer {}",
                    self.id,
                    peer_id
                );
                peer_addr.do_send(Eleccion(self.id));
                envio_a_mayores = true;
            }
        }

        if !envio_a_mayores {
            // No hay nodos con ID mayor, me declaro líder
            log_info!(
                self.logger,
                "YpfRuta {}: No hay nodos con ID mayor. Me declaro líder.",
                self.id
            );
            self.declarar_lider(ctx);
        } else {
            // Esperar timeout para respuestas (2 segundos)
            ctx.run_later(Duration::from_secs(2), |_act, ctx| {
                ctx.address().do_send(EleccionTimeout);
            });
        }
    }
}
