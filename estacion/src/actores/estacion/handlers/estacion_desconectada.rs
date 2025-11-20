use actix::{Handler, Context, AsyncContext, ActorFutureExt, WrapFuture};
use crate::actores::estacion::Estacion;
use crate::actores::estacion_cercana::Enviar;
use crate::actores::estacion::messages::{EstacionDesconectada, Reenviar};

impl Handler<EstacionDesconectada> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: EstacionDesconectada, ctx: &mut Context<Self>) {
        if self.desconectada {
            // Si ya estamos desconectados, no hacemos nada.

            return;
        }

        println!("La estación {} se ha desconectado.", msg.estacion_id);
        self.estaciones_cercanas.remove(&msg.estacion_id);

        println!("[{}] ❌ sin conexión a {}, intentando reconectar...", self.id, msg.estacion_id);

        println!("[{}] ❌ sin conexión a {}, hay {} estaciones cercanas...", self.id, msg.estacion_id, self.todas_las_estaciones.len());

        //primer reintento
        self.siguiente_estacion = if self.id + 1 >= self.todas_las_estaciones.len() {
            0
        } else {
            self.id + 1
        };

        let proxima = self.todas_las_estaciones.get(&msg.estacion_id).unwrap().clone();
        let addr_self = ctx.address();
        let self_id = self.id;
        let mensaje_clone = msg.mensaje.clone();
        let current_id = msg.estacion_id;
        self.siguiente_estacion = msg.estacion_id;

        let prox_id = if self.siguiente_estacion + 1 >= self.todas_las_estaciones.len() {
            0
        } else {
            self.siguiente_estacion + 1
        };

        ctx.spawn(
            actix::fut::wrap_future({
                let proxima = proxima.clone();
                let addr_self2 = addr_self.clone();
                let mensaje2 = mensaje_clone.clone();

                async move {
                    if Estacion::intentar_conectar(
                        proxima,
                        addr_self2.clone(),
                        self_id,
                        current_id
                    ).await.is_ok() {
                        println!("Reconexión exitosa desde Desconectada");
                        Some(mensaje2)
                    } else {
                        println!("Reconexion fallida, reintentaremos con {}", prox_id);
                        addr_self2.do_send(EstacionDesconectada {
                            estacion_id: prox_id,
                            mensaje: mensaje2,
                        });
                        None
                    }
                }
            })
                .map(move |maybe_msg, act: &mut Estacion, ctx: &mut Context<Estacion>| {

                    if let Some(mensaje) = maybe_msg {
                        println!("Enviando mensaje desde Desconectada con id: {}", &current_id);
                        ctx.address().do_send(Reenviar{ bytes: mensaje})
                    }
                }),
        );
    }
}