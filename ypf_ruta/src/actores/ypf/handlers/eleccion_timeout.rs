use crate::actores::ypf::messages::EleccionTimeout;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Context, Handler};
use util::log_info;

impl Handler<EleccionTimeout> for YpfRuta {
    type Result = ();

    fn handle(&mut self, _msg: EleccionTimeout, ctx: &mut Context<Self>) -> Self::Result {
        if !self.en_eleccion {
            return;
        }

        if self.respuestas_recibidas == 0 {
            log_info!(
                self.logger,
                "YpfRuta {}: Timeout alcanzado sin respuestas. Me declaro líder.",
                self.id
            );
            self.declarar_lider(ctx);
        } else {
            log_info!(
                self.logger,
                "YpfRuta {}: Recibí {} respuestas OK. Esperando anuncio del nuevo líder.",
                self.id,
                self.respuestas_recibidas
            );
            self.en_eleccion = false;
        }
    }
}
