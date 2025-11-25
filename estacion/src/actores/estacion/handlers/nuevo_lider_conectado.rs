use std::time::Duration;
use crate::actores::estacion::{EnviarVentasAgrupadas, Estacion, InformarVenta, NuevoLiderConectado};
use crate::actores::estacion_cercana::Enviar;
use actix::{AsyncContext, Handler, WrapFuture};
use tokio::time::sleep;

impl Handler<NuevoLiderConectado> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: NuevoLiderConectado, ctx: &mut Self::Context) -> Self::Result {
        println!(
            "[{}] Nuevo líder conectado, Enviando ventas pendientes de confirmacion",
            self.id,
        );

        if self.lider_actual == Some(self.id) {
            println!(
                "[{}] Soy el líder actual, me envio a mi mismo las ventas pendientes de confirmacion",
                self.id,
            );

            if let Some(_ventas_pendientes) = self.ventas_por_informar.get(&self.id) {
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
            // como soy líder, no reenvío a ningún otro líder
            return;
        }

        // Si NO soy el líder (reenvío al líder nuevo)
        if let Some(ventas_pendientes) = self.ventas_por_informar.get(&self.id) {
            if let Some(lider) = self.buscar_estacion_lider() {
                for (id_surtidor, ventas) in ventas_pendientes {
                    for venta in ventas {
                        let venta = venta.clone();
                        let mensaje = InformarVenta {
                            venta,
                            id_surtidor: *id_surtidor,
                            id_estacion: self.id,
                        };
                        lider.do_send(Enviar {
                            bytes: mensaje.to_bytes(),
                        });
                    }
                }
            }
        }
        self.ventas_por_informar.clear();
        self.limpiar_ventas_sin_informar();
    }
}
