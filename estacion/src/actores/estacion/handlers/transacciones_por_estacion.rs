use actix::{Context, Handler};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::TransaccionesPorEstacion;
use crate::actores::estacion::messages::ConfirmarTransacciones;
use std::collections::HashMap;

impl Handler<TransaccionesPorEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: TransaccionesPorEstacion, _ctx: &mut Context<Self>) {
        println!("[{}] Soy el lider y recibi las transacciones para confirmar: {:?}", self.id, msg.transacciones);
        // separo las ventas confirmadas por estación y se las mando a cada una
        for (id_estacion, surtidores) in &msg.transacciones {
            let mut resultados_estacion: HashMap<usize, Vec<(usize, bool)>> = HashMap::new();

            for (id_surtidor, ventas) in surtidores {
                let mut resultados_surtidor = Vec::new();

                for venta in ventas {
                    // Validación dummy (true)
                    resultados_surtidor.push((venta.0, venta.1));
                }

                resultados_estacion.insert(*id_surtidor, resultados_surtidor);
            }
            self.estaciones_cercanas.get(id_estacion).unwrap().do_send(ConfirmarTransacciones {
                transacciones: resultados_estacion,
            });
        }

    }
}