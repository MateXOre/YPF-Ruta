use std::collections::HashMap;
use std::time::Duration;
use actix::{AsyncContext, Context, Handler, WrapFuture};
use tokio::time::sleep;
use crate::actores::estacion::{EnviarVentasAgrupadas, Estacion};
use crate::actores::estacion::messages::CobrarACliente;
use crate::actores::estacion::messages::InformarVenta;
use crate::actores::estacion_cercana::Enviar;

impl Handler<CobrarACliente> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: CobrarACliente, ctx: &mut Context<Self>) {
        println!("[{}] Cobranza informada: {:?} por surtidor: {}", self.id, msg.venta.id_venta, msg.surtidor_id);

        if self.lider_actual == Some(self.id) {

            if self.ventas_por_informar.is_empty() {
                // iniciar temporizador
                if !self.temporizador_activo {
                    self.temporizador_activo = true;

                    let addr = ctx.address();
                    ctx.spawn(
                        async move {
                            sleep(Duration::from_secs(10)).await;
                            addr.do_send(EnviarVentasAgrupadas);
                        }
                            .into_actor(self)
                    );
                }
            }

            // Soy líder
            self.ventas_por_informar
                .entry(self.id)
                .or_insert_with(HashMap::new)
                .entry(msg.surtidor_id)
                .or_insert_with(Vec::new)
                .push(msg.venta);
        } else {
            // NO soy líder → envío venta al líder
            if let Some(lider_addr) = self.buscar_estacion_lider() {

                let venta = InformarVenta {
                    venta: msg.venta,
                    id_surtidor: msg.surtidor_id,
                    id_estacion: self.id,
                };
                lider_addr.do_send(Enviar{bytes: venta.to_bytes()});
            }
        }

        /*let surtidor = self.surtidores.get(&msg.surtidor_id);

        surtidor.unwrap().do_send(ResultadoVenta{exito: true, id_venta : msg.venta.id_venta}) //de momento la confirma unilateralmente(parecido a lo que debería hacer si esta offline)*/

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