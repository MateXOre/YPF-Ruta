use crate::actores::estacion::{DesconexionDetectada, Eleccion, Estacion, EstacionDesconectada};
use crate::actores::estacion_cercana::Enviar;
use actix::{AsyncContext, Handler};
use util::log_info;

impl Handler<DesconexionDetectada> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: DesconexionDetectada, ctx: &mut Self::Context) -> Self::Result {
        if let Some(lider_id) = self.lider_actual {
            if lider_id == msg.estacion_id {
                log_info!(
                    self.logger,
                    "[{}] Desconexión detectada del líder actual: {}. Actualizando estado.",
                    self.id,
                    msg.estacion_id
                );
                self.lider_actual = None;

                for (id_surtidor, venta) in self.ventas_a_confirmar.iter() {
                    log_info!(self.logger, "[{}] Venta del surtidor {} no se pude confirmar debido a la desconexión del líder. Iniciando elección de lider", self.id, id_surtidor);

                    self.ventas_por_informar
                        .entry(self.id)
                        .or_default()
                        .entry(*id_surtidor)
                        .or_default()
                        .push(venta.clone());
                }
                self.ventas_a_confirmar.clear();
                log_info!(
                    self.logger,
                    "[{}] Iniciando elección de nuevo líder",
                    self.id
                );

                let mensaje = Eleccion {
                    aspirantes_ids: vec![self.id],
                };

                let id = self.id;
                let siguiente_estacion = self.siguiente_estacion;

                if let Some(siguiente) = self
                    .estaciones_cercanas
                    .get(&self.siguiente_estacion)
                    .cloned()
                {
                    let logger = self.logger.clone();
                    actix_rt::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        siguiente.do_send(Enviar {
                            bytes: mensaje.to_bytes(),
                        });
                        log_info!(
                            logger,
                            "[{}] Enviando mensaje inicial a estación {}",
                            id,
                            siguiente_estacion
                        );
                    });
                } else {
                    log_info!(
                        self.logger,
                        "[{}] Estación {} no conectada, reintentando conexión",
                        id,
                        siguiente_estacion
                    );

                    ctx.address().do_send(EstacionDesconectada {
                        estacion_id: self.siguiente_estacion,
                        mensaje: mensaje.to_bytes(),
                    });
                }
            } else {
                log_info!(
                    self.logger,
                    "[{}] Desconexión detectada de la estación {} que no es líder actual.",
                    self.id,
                    msg.estacion_id
                );
            }
        }
    }
}
