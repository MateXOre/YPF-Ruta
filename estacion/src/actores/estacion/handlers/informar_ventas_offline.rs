use crate::actores::estacion::messages::*;
use crate::actores::estacion::Estacion;
use actix::{AsyncContext, Context, Handler, WrapFuture};
use util::log_info;
use std::time::Duration;
use tokio::time::sleep;

impl Handler<InformarVentasOffline> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: InformarVentasOffline, ctx: &mut Context<Self>) {
        log_info!(self.logger, "[{}] Informar ventas offline a lider: {}", self.id, msg.id_lider);
        if !self.estoy_conectada {
            self.estoy_conectada = true;
            self.lider_actual = Some(msg.id_lider);
        }
        if self.lider_actual.is_none() {
            self.lider_actual = Some(msg.id_lider);
        }
        if self.id == msg.id_lider {
            log_info!(self.logger, "[{}] Soy el lider, y me guardo las ventas offline acumuladas para informar a YPF RUTA", self.id);
            if self.ventas_por_informar.is_empty()
                && !msg.ventas.is_empty()
                && !self.temporizador_activo
            {
                self.temporizador_activo = true;

                let addr = ctx.address();
                ctx.spawn(
                    async move {
                        sleep(Duration::from_secs(10)).await;
                        addr.do_send(EnviarVentasAgrupadas);
                    }
                    .into_actor(self),
                );
            }
            self.ventas_por_informar = self.agregar_ventas_acumuladas(msg.ventas);
            self.guardar_ventas_sin_informar();
        } else {
            log_info!(self.logger, "[{}] Soy no lider, y sigo ronda de informar ventas offline", self.id);

            // Combinar las ventas de esta estación con las acumuladas
            let ventas_acumuladas = self.agregar_ventas_acumuladas(msg.ventas);
            if self.ventas_por_informar.is_empty() {
                log_info!(self.logger, "[{}] No tengo ventas acumuladas offline", self.id);
            } else {
                self.ventas_por_informar.clear();
                self.limpiar_ventas_sin_informar();
                log_info!(self.logger, "[{}] Tengo ventas acumuladas offline", self.id);
            }

            let mensaje_bytes = InformarVentasOffline {
                id_lider: msg.id_lider,
                ventas: ventas_acumuladas,
            }
            .to_bytes();
            self.enviar_a_siguiente(ctx, mensaje_bytes);
        }
    }
}
