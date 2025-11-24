use actix::{ActorContext, ActorFutureExt};
use actix::{AsyncContext, Context, Handler, WrapFuture};
use tokio::io::AsyncWriteExt;
use crate::actores::estacion::estacion_actor::Estacion;
use crate::actores::estacion::messages::{Resultado, ResultadoVentas};

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
        println!("Estacion: recibí ResultadoVentas con {} ventas.", msg.ventas.len());

        if let Some(mut socket) = self.socket.take() {
            let fut = async move {
                let bytes = parse_to_json(msg.ventas);
                if socket.write_all(&*bytes).await.is_err() {
                    eprintln!("Estacion: error serializando ResultadoVentas");
                }
            };

            ctx.spawn(
                fut.into_actor(self)
                    .map(|_, _act, ctx| {
                        ctx.stop();
                    }),
            );
        } else {
            println!("Estacion: no hay socket para enviar ResultadoVentas, finalizando actor.");
            ctx.stop();
        }
    }
}