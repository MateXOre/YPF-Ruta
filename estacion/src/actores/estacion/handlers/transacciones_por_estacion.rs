use actix::{Context, Handler};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::TransaccionesPorEstacion;
use crate::actores::estacion::messages::ConfirmarTransacciones;
use std::collections::HashMap;
use util::structs::venta::Venta;

impl Handler<TransaccionesPorEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: TransaccionesPorEstacion, _ctx: &mut Context<Self>) {
        println!("[{}] Soy el lider y recibi las transacciones para confirmar: {:?}", self.id, msg.transacciones);

        // separo las ventas confirmadas por estación

        // let mut ventas_por_estacion: HashMap<usize, Vec<Venta>> = HashMap::new();

        // // Talvez no pasar Venta entera sino una especie de comprobante de la venta por la correccion del profe.
        // for venta in msg.transacciones {
        //     ventas_por_estacion.entry(venta.id_estacion).or_insert(vec![]).push(venta);
        // }

        // ventas_por_estacion.iter().for_each(|(id_estacion, ventas)| {
        //     if let Some(addr) = self.buscar_estacion_por_id(*id_estacion) {
        //         addr.do_send(ConfirmarTransacciones {
        //             transacciones: ventas.clone(),
        //         });
        //     }

        // });
    }
}