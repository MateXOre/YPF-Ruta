use actix::{Handler, Context, AsyncContext, WrapFuture};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use tokio::time::sleep;
use std::time::Duration;


impl Handler<InformarVentasOffline> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: InformarVentasOffline, ctx: &mut Context<Self>) {
        println!("[{}] Informar ventas offline a lider: {}", self.id, msg.id_lider);

        if self.id == msg.id_lider {
            println!("[{}] Soy el lider, y me guardo las ventas offline acumuladas para informar a YPF RUTA", self.id);
            let ventas_acumuladas = self.agregar_ventas_acumuladas(msg.ventas);
            self.ventas_por_informar = ventas_acumuladas;


            if !self.temporizador_ventas_offline_activo {
                self.temporizador_ventas_offline_activo = true;
                let addr = ctx.address();
                ctx.spawn(
                    async move {
                        sleep(Duration::from_secs(Estacion::TIEMPO_INFORMAR_VENTAS_OFFLINE)).await;
                        addr.do_send(EmpezarInformarVentasOffline{});
                    }
                    .into_actor(self)
                );
            }
        } else {
            println!("[{}] Soy no lider, y sigo ronda de informar ventas offline", self.id);
            
            // Combinar las ventas de esta estación con las acumuladas
            let ventas_acumuladas = self.agregar_ventas_acumuladas(msg.ventas);
            if self.ventas_por_informar.is_empty() {
                println!("[{}] No tengo ventas acumuladas offline", self.id);
            } else {
                self.ventas_por_informar.clear();
                println!("[{}] Tengo ventas acumuladas offline", self.id);
            }
            
            let mensaje_bytes = InformarVentasOffline{id_lider: msg.id_lider, ventas: ventas_acumuladas}.to_bytes();
            self.enviar_a_siguiente(ctx, mensaje_bytes);
        }
    }
}