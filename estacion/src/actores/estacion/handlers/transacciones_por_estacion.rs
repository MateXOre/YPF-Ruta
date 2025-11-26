use crate::actores::estacion::messages::ConfirmarTransacciones;
use crate::actores::estacion::messages::TransaccionesPorEstacion;
use crate::actores::estacion::Estacion;
use crate::actores::estacion_cercana::Enviar;
use actix::{AsyncContext, Context, Handler};
use util::log_info;
use std::collections::HashMap;

impl Handler<TransaccionesPorEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: TransaccionesPorEstacion, ctx: &mut Context<Self>) {
        log_info!(
            self.logger,
            "[{}] Soy el lider y recibi las transacciones para confirmar: {:?}",
            self.id,
            msg.transacciones
        );
        // separo las ventas confirmadas por estación y se las mando a cada una
        for (id_estacion, surtidores) in &msg.transacciones {
            let mut resultados_estacion: HashMap<usize, Vec<(usize, bool)>> = HashMap::new();

            for (id_surtidor, ventas) in surtidores {
                let mut resultados_surtidor = Vec::new();

                for venta in ventas {
                    resultados_surtidor.push((venta.0, venta.1));
                }

                resultados_estacion.insert(*id_surtidor, resultados_surtidor);
            }

            let transacciones_confirmadas = ConfirmarTransacciones {
                transacciones: resultados_estacion,
            };
            if let Some(estacion_addr) = self.estaciones_cercanas.get(id_estacion) {
                log_info!(self.logger, "[{}] Mando las transacciones a la estación {}", self.id, id_estacion);
                estacion_addr.do_send(Enviar {
                    bytes: transacciones_confirmadas.to_bytes(),
                });
            } else if *id_estacion == self.id {
                log_info!(self.logger, "[{}] Me mando las transacciones a mi mismo porque soy el lider", self.id);
                ctx.address().do_send(transacciones_confirmadas);
            } else {
                log_info!(self.logger, "[{}] Ignorando transacciones para estación inexistente {}", self.id, id_estacion);
            }
        }
    }
}
