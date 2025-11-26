use crate::actores::estacion::messages::*;
use crate::actores::estacion::Estacion;
use actix::prelude::*;
use actix::ActorFutureExt;
use actix::{Context, Handler};
use util::log_error;
use util::log_info;
use std::time::Duration;
use tokio::time::sleep;

impl Handler<NotificarLider> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: NotificarLider, ctx: &mut Context<Self>) {
        if self.id == msg.id_iniciador && self.lider_actual == Some(msg.id_lider) {
            log_info!(self.logger, "[{}] Mensaje de líder {} completó el ciclo, fin de propagación.", self.id, msg.id_lider);
            return;
        }

        if let Some(lider) = self.lider_actual {
            if lider == self.id {
                log_info!(self.logger, "[{}] Soy el líder, entonces no es necesario que siga el anillo", self.id);
                return;
            }
        }

        self.lider_actual = Some(msg.id_lider);

        log_info!(self.logger, "[{}] Mi nuevo lider es : {} ", self.id, msg.id_lider);

        if self.id != msg.id_lider {
            if self.estaciones_cercanas.contains_key(&msg.id_lider) {
                ctx.address().do_send(NuevoLiderConectado);
            } else if let Some(lider_addr) = self.todas_las_estaciones.get(&msg.id_lider).copied() {
                let addr_self = ctx.address();
                let self_id = self.id;
                let nuevo_lider = msg.id_lider;

                let logger = self.logger.clone();
                ctx.spawn(
                    actix::fut::wrap_future(async move {
                        match Estacion::intentar_conectar(
                            lider_addr,
                            addr_self.clone(),
                            self_id,
                            nuevo_lider,
                            logger.clone(),
                        )
                        .await
                        {
                            Ok(_) => {
                                addr_self.do_send(NuevoLiderConectado);
                            }
                            Err(_) => log_error!(logger, "[{}] no se pudo conectar con el líder {}", self_id, nuevo_lider),
                        }
                    })
                    .map(|_, _, _| ()),
                );
            }
        } else {
            log_info!(self.logger, "[{}] Soy el nuevo líder, no necesito conectarme a mí mismo.", self.id);
            ctx.address().do_send(NuevoLiderConectado);
            let addr = ctx.address();
            ctx.spawn(
                async move {
                    sleep(Duration::from_secs(
                        Estacion::TIEMPO_INFORMAR_VENTAS_OFFLINE,
                    ))
                    .await;
                    addr.do_send(EmpezarInformarVentasOffline {});
                }
                .into_actor(self),
            );
        }

        let mensaje_serializado = NotificarLider {
            id_lider: msg.id_lider,
            id_iniciador: msg.id_iniciador,
        }
        .to_bytes();
        self.enviar_a_siguiente(ctx, mensaje_serializado);
    }
}
