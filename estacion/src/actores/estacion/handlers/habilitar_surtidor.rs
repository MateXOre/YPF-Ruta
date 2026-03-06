use crate::actores::estacion::{Estacion, HabilitarSurtidor};
use actix::{Context, Handler};
use util::log_info;

impl Handler<HabilitarSurtidor> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: HabilitarSurtidor, _ctx: &mut Context<Self>) {
        log_info!(
            self.logger,
            "[{}] Habilitando surtidor: {}",
            self.id,
            msg.surtidor_id
        );
        self.surtidores.insert(msg.surtidor_id, msg.surtidor_addr);
    }
}
