use actix::{Context, Handler};
use crate::actores::estacion::{ResultadoVentas, Estacion};
use crate::actores::surtidor::messages::ResultadoVenta;

impl Handler<ResultadoVentas> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVentas, _ctx: &mut Context<Self>) {
        for (id_surtidor, datos_estacion) in msg.resultados{
            if let Some(surtidor) = self.surtidores.get(&id_surtidor) {
                surtidor.do_send(ResultadoVenta { exito: true });
            } else {
                println!(
                    "[Estación {}] Ignorando resultado para surtidor inexistente {}",
                    self.id, id_surtidor
                );
            }
        }
    }
}