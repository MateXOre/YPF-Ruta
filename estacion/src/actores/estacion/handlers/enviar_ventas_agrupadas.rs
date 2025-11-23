use std::collections::HashMap;
use actix::{Context, Handler, AsyncContext};
use crate::actores::estacion::{EnviarVentasAgrupadas, Estacion};
use crate::actores::estacion::messages::TransaccionesPorEstacion;

impl Handler<EnviarVentasAgrupadas> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: EnviarVentasAgrupadas, ctx: &mut Context<Self>) {
        // Recorro ventas agrupadas
        // simula que valido las ventas y devuelvo el resultado de la venta (como si fuese ypf ruta)
        let mut resultados_estaciones: HashMap<usize, HashMap<usize, Vec<(usize, bool)>>> = HashMap::new();
        for (id_estacion, surtidores) in &self.ventas_por_informar {
            let mut resultados_estacion: HashMap<usize, Vec<(usize, bool)>> = HashMap::new();

            for (id_surtidor, ventas) in surtidores {
                let mut resultados_surtidor = Vec::new();

                for venta in ventas {
                    // Validación dummy (true)
                    resultados_surtidor.push((venta.id_venta, true));
                }

                resultados_estacion.insert(*id_surtidor, resultados_surtidor);
            }

            resultados_estaciones.insert(*id_estacion, resultados_estacion);
        }

        // simulo que esto me llega desde YPFRuta
        ctx.address().do_send(TransaccionesPorEstacion {
            transacciones: resultados_estaciones,
        });
        self.ventas_por_informar.clear();
        self.temporizador_activo = false;
    }
}