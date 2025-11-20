use actix::{Context, Handler, AsyncContext};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::InformarVenta;
use crate::actores::estacion::messages::TransaccionesPorEstacion;

impl Handler<InformarVenta> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: InformarVenta, _ctx: &mut Context<Self>) {
        println!("[{}] Soy el lider y recibi la venta para confirmar: {:?}", self.id, msg.venta.id_venta);

        // Acá se guardaria la data para luego enviarla a YPfRUTA

        // Por ahora para simular que le mando el mensaje a YPfRUTA y YPfRUTA me responde,
        // me mando el mensaje transacciones_por_estacion a mi mismo.
        
        // ctx.address().do_send(TransaccionesPorEstacion {
        //     transacciones: vec![msg.venta],
        // });

    }
}