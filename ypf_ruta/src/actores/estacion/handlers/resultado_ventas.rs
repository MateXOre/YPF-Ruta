use crate::actores::estacion::estacion_actor::Estacion;
use crate::actores::estacion::messages::{Resultado, ResultadoVentas};
use actix::{ActorContext, ActorFutureExt};
use actix::{AsyncContext, Context, Handler, WrapFuture};
use tokio::io::AsyncWriteExt;
use util::{log_debug, log_error};

fn parse_to_json(ventas: Resultado) -> Vec<u8> {
    match serde_json::to_vec(&ventas) {
        Ok(mut bytes) => {
            bytes.push(b'\n');
            bytes
        }
        Err(e) => {
            eprintln!("Estacion: error serializando ResultadoVentas: {}", e);
            Vec::new()
        }
    }
}

impl Handler<ResultadoVentas> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVentas, ctx: &mut Context<Self>) -> Self::Result {
        log_debug!(self.logger, "Estacion: recibí ResultadoVentas");
        if let Some(mut socket) = self.socket.take() {
            let logger = self.logger.clone();
            let fut = async move {
                let bytes = parse_to_json(msg.ventas);
                if socket.write_all(&*bytes).await.is_err() {
                    log_error!(logger, "Estacion: error serializando ResultadoVentas");
                }
            };

            ctx.spawn(fut.into_actor(self).map(|_, _act, ctx| {
                ctx.stop();
            }));
        } else {
            log_debug!(self.logger, "Estacion: no hay socket para enviar ResultadoVentas, finalizando actor.");
            ctx.stop();
        }
    }
}
