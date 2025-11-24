use crate::actores::estacion::messages::LiberarClientesEnCola;
use crate::actores::estacion::Estacion;
use crate::actores::surtidor::messages::ResultadoVenta;
use actix::{Context, Handler};

impl Handler<LiberarClientesEnCola> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: LiberarClientesEnCola, _ctx: &mut Context<Self>) {
        println!("[{}] Liberando clientes en cola", self.id);
        for (surtidor_id, venta) in self.ventas_a_confirmar.iter() {
            // pasamos las ventas online a offline
            let mut venta = venta.clone();
            venta.offline = true;
            self.ventas_por_informar
                .entry(venta.id_estacion)
                .or_default()
                .entry(*surtidor_id)
                .or_default()
                .push(venta);
        }
        self.ventas_a_confirmar.clear();
        for surtidor in self.surtidores.values() {
            // avisamos a los surtidores que liberamos a los clientes
            surtidor.do_send(ResultadoVenta { exito: true });
        }
    }
}
