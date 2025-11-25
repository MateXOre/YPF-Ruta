use crate::actores::ypf::messages::NuevoLider;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Context, Handler};
use util::{log_debug, log_info};

impl Handler<NuevoLider> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: NuevoLider, ctx: &mut Context<Self>) -> Self::Result {
        log_debug!(
            self.logger,
            "YpfRuta {}: Recibido anuncio de COORDINATOR del nodo {}",
            self.id,
            msg.id
        );

        if self.id == msg.id {
            log_info!(self.logger, "YpfRuta {}: He sido elegido líder.", self.id);
        } else {
            log_info!(
                self.logger,
                "YpfRuta {}: Nodo {} es el nuevo líder.",
                self.id,
                msg.id
            );
        }
        self.lider = Some(msg.id);
        self.en_eleccion = false;
        self.respuestas_recibidas = 0;

        if self.lider == Some(self.id) {
            self.escuchar_estaciones(ctx)
        }
    }
}
