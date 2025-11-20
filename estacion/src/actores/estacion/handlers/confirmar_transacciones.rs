use actix::{Context, Handler};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::ConfirmarTransacciones;
use crate::actores::surtidor::messages::ResultadoVenta;
use util::structs::venta::EstadoVenta;

impl Handler<ConfirmarTransacciones> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ConfirmarTransacciones, _ctx: &mut Context<Self>) {
        println!("[{}] Soy una estación y recibi la confirmación de transacciones: {:?}", self.id, msg.transacciones);

        // msg.transacciones.iter().for_each(|venta| {
        //     if let Some(surtidor_id) = self.ventas_a_confirmar.get(&venta.id_venta) {
        //         if let Some(addr) = self.surtidores.get(surtidor_id) {
        //             addr.do_send(ResultadoVenta {
        //                 exito: venta.estado == EstadoVenta::Confirmada,
        //                 id_venta: venta.id_venta,
        //             });
        //         }
        //     }
        // });

    }
}