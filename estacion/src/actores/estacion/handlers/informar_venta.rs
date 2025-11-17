use actix::{Context, Handler};
use crate::actores::{estacion::Estacion, surtidor::messages::{InformarVenta, ResultadoVenta}};

impl Handler<InformarVenta> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: InformarVenta, ctx: &mut Context<Self>) {
        println!("[{}] Venta informada: {:?}", self.id, msg.venta.id_venta);

        // Acá se envía todo a lider/YPfRUTA

        self.surtidores.iter().find(|(id, addr)| **id == msg.surtidor_id).map(|(_, addr)| {
            addr.do_send(ResultadoVenta {
                exito: true,
                id_venta: msg.venta.id_venta,
            });
        });
    }
}