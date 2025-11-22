use actix::{Context, Handler};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::CobrarACliente;
use crate::actores::estacion::messages::InformarVenta;
use crate::actores::surtidor::messages::ResultadoVenta;

impl Handler<CobrarACliente> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: CobrarACliente, _ctx: &mut Context<Self>) {
        println!("[{}] Cobranza informada: {:?} por surtidor: {}", self.id, msg.venta.id_venta, msg.surtidor_id);
/*
        if self.lider_actual == Some(self.id) {
            // Soy líder: respondo directo por ahora
            if let Some(surtidor) = self.surtidores.get(&msg.surtidor_id) {
                surtidor.do_send(ResultadoVenta {
                    exito: true,
                    id_venta: msg.venta.id_venta,
                });
            }
        } else {
            // NO soy líder → envío venta al líder
            if let Some(lider_addr) = self.buscar_estacion_lider() {

                /*lider_addr.do_send(InformarVenta {
                    venta: msg.venta,
                    surtidor_id: msg.surtidor_id,
                    estacion_origen: self.id,
                });*/
            }
        }*/

        let surtidor = self.surtidores.get(&msg.surtidor_id);

        surtidor.unwrap().do_send(ResultadoVenta{exito: true, id_venta : msg.venta.id_venta}) //de momento la confirma unilateralmente(parecido a lo que debería hacer si esta offline)

        // if self.lider_actual != Some(self.id) {

        //     self.ventas_a_confirmar.insert(msg.venta.id_venta, msg.surtidor_id);
        //     if let Some(addr) = self.buscar_estacion_lider() {
        //         addr.do_send(InformarVenta {
        //             venta: msg.venta
        //         });
        //     }
        // }*/
    }
}