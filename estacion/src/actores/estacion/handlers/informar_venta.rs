use crate::actores::estacion::messages::InformarVenta;
use crate::actores::estacion::{EnviarVentasAgrupadas, Estacion};
use actix::{AsyncContext, Context, Handler, WrapFuture};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

impl Handler<InformarVenta> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: InformarVenta, ctx: &mut Context<Self>) {
        println!(
            "[{}] Soy el lider y recibi la venta de {} para confirmar",
            self.id, msg.id_estacion
        );

        if self.ventas_por_informar.is_empty() {
            // iniciar temporizado
            if !self.temporizador_activo {
                self.temporizador_activo = true;
                println!("Iniciando temporizador para informar ventas agrupadas");

                let addr = ctx.address();
                ctx.spawn(
                    async move {
                        sleep(Duration::from_secs(10)).await;
                        addr.do_send(EnviarVentasAgrupadas);
                    }
                    .into_actor(self),
                );
            }
        }

        self.ventas_por_informar
            .entry(msg.id_estacion)
            .or_insert_with(HashMap::new)
            .entry(msg.id_surtidor)
            .or_insert_with(Vec::new)
            .push(msg.venta);
    }
}
