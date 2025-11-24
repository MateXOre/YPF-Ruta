use crate::actores::estacion::messages::ConfirmarTransacciones;
use crate::actores::estacion::Estacion;
use crate::actores::surtidor::messages::ResultadoVenta;
use actix::{Context, Handler};

impl Handler<ConfirmarTransacciones> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ConfirmarTransacciones, _ctx: &mut Context<Self>) {
        println!(
            "[{}] Soy una estación y recibi la confirmación de transacciones: {:?}",
            self.id, msg.transacciones
        );
        for (id_surtidor, resultados_ventas) in msg.transacciones {
            if let Some(_venta) = self.ventas_a_confirmar.get(&id_surtidor) {
                self.ventas_a_confirmar.remove(&id_surtidor);
            } // borro las ventas no offline que tenia a confirmar
            if let Some(surtidor) = self.surtidores.get(&id_surtidor) {
                if let Some(resultado_venta) = resultados_ventas.first() {
                    surtidor.do_send(ResultadoVenta {
                        exito: resultado_venta.1,
                    });
                } else {
                    println!(
                        "[Estación {}] Ignorando resultado para surtidor {}",
                        self.id, id_surtidor
                    );
                }
            } else {
                println!(
                    "[Estación {}] Ignorando resultado para surtidor inexistente {}",
                    self.id, id_surtidor
                );
            }
        }
    }
}
