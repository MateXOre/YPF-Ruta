use crate::actores::estacion::SurtidorLibre;
use crate::actores::surtidor::messages::*;
use crate::actores::surtidor::surtidor::Surtidor;
use actix::prelude::*;
use actix::{Context, Handler};
use util::log_info;

impl Handler<Detenerme> for Surtidor {
    type Result = ();

    fn handle(&mut self, _msg: Detenerme, ctx: &mut Context<Self>) {
        log_info!(
            self.logger,
            "[{}] Deteniendo Surtidor {}",
            self.estacion_id,
            self.id
        );
        self.estacion.do_send(SurtidorLibre {
            surtidor_id: self.id,
        });

        let _ = self.writer_tx.send(Vec::new());
        if let Some(reader) = self.reader.take() {
            drop(reader);
        }
        ctx.stop();
    }
}
