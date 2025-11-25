use crate::actores::estacion::estacion_actor::ValidarVentas;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Context, Handler};
use util::log_debug;

impl Handler<ValidarVentas> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: ValidarVentas, _ctx: &mut Context<Self>) -> Self::Result {
        log_debug!(
            self.logger,
            "YpfRuta {}: Recibidas {} ventas para validar",
            self.id,
            msg.ventas.len()
        );

        self.ventas_por_confirmar.push_back((msg.from, msg.ventas));

        log_debug!(
            self.logger,
            "YpfRuta {}: Ventas agregadas a la cola. Total en cola: {}",
            self.id,
            self.ventas_por_confirmar.len()
        );
    }
}
