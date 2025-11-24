use actix::{Context, Handler};
use crate::actores::estacion::{Estacion};
use crate::actores::surtidor::messages::ResultadoVenta;
use crate::actores::estacion::messages::LiberarClientesEnCola;

impl Handler<LiberarClientesEnCola> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: LiberarClientesEnCola, _ctx: &mut Context<Self>) {
        println!("[{}] Liberando clientes en cola", self.id);
        for surtidor in self.surtidores.values() {
            surtidor.do_send(ResultadoVenta {
                exito: true,
            });
        }
    }
}