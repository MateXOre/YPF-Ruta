use crate::actores::estacion::messages::ConfirmarTransacciones;
use crate::actores::estacion::Estacion;
use crate::actores::surtidor::messages::ResultadoVenta;
use actix::{Context, Handler};
use util::log_info;

impl Handler<ConfirmarTransacciones> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ConfirmarTransacciones, _ctx: &mut Context<Self>) {
        log_info!(
            self.logger,
            "[{}] Soy una estación y recibi la confirmación de transacciones: {:?}",
            self.id,
            msg.transacciones
        );
        for (id_surtidor, resultados_ventas) in msg.transacciones {
            if let Some(_venta) = self.ventas_a_confirmar.get(&id_surtidor) {
                self.ventas_a_confirmar.remove(&id_surtidor);
            }
            if let Some(surtidor) = self.surtidores.get(&id_surtidor) {
                if let Some(resultado_venta) = resultados_ventas.first() {
                    surtidor.do_send(ResultadoVenta {
                        exito: resultado_venta.1,
                    });
                } else {
                    log_info!(
                        self.logger,
                        "[Estación {}] Ignorando resultado para surtidor {}",
                        self.id,
                        id_surtidor
                    );
                }
            } else {
                log_info!(
                    self.logger,
                    "[Estación {}] Ignorando resultado para surtidor inexistente {}",
                    self.id,
                    id_surtidor
                );
            }
        }
    }
}
