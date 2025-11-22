use std::collections::HashMap;
use actix::{Context, Handler};
use util::structs::venta::EstadoVenta::Confirmada;
use crate::actores::estacion::{EnviarVentasAgrupadas, Estacion, ResultadoVentas};
use crate::actores::estacion_cercana::Enviar;
use crate::actores::surtidor::messages::ResultadoVenta;

impl Handler<EnviarVentasAgrupadas> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: EnviarVentasAgrupadas, _ctx: &mut Context<Self>) {
        // Recorro ventas agrupadas
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

            // Construyo el mensaje final para la estación
            let mensaje = ResultadoVentas {
                resultados: resultados_estacion,
            }
                .to_bytes();

            // Envío a estación correspondiente
            if let Some(estacion_addr) = self.estaciones_cercanas.get(id_estacion) {
                estacion_addr.do_send(Enviar{ bytes: mensaje});
            }
        }

        self.ventas_por_informar.clear();
        self.temporizador_activo = false;
    }
}