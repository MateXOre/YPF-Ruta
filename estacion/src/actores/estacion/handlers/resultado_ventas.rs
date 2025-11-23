use actix::{Context, Handler};
use crate::actores::estacion::{ResultadoVentas, Estacion};
use crate::actores::surtidor::messages::ResultadoVenta;

impl Handler<ResultadoVentas> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVentas, _ctx: &mut Context<Self>) {
         // despues iterar por resultados_ventas para quitar las ventas offline de la estacion
        for (id_surtidor, resultados_ventas) in msg.resultados{
            if let Some(surtidor) = self.surtidores.get(&id_surtidor) {
                if let Some(resultado_venta) = resultados_ventas.first() {
                    surtidor.do_send(ResultadoVenta { exito: resultado_venta.1 });
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