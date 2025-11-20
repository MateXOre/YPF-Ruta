use actix::{Handler, Context, AsyncContext, ActorFutureExt, WrapFuture};
use crate::actores::estacion::Estacion;
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

        //primer reintento
        let proxima = self.todas_las_estaciones.get(&msg.estacion_id).unwrap().clone();
        let addr_self = ctx.address();
        let self_id = self.id;
        let mensaje_clone = msg.mensaje.clone();
        let current_id = self.siguiente_estacion;

        let prox_id = if self.siguiente_estacion + 1 >= self.estaciones_cercanas.len() {
            0
        } else {
            self.siguiente_estacion + 1
        };
        
        ctx.spawn(
            actix::fut::wrap_future(async move {
                if Estacion::intentar_conectar(proxima, addr_self.clone(), self_id, current_id).await.is_ok() {
                    Some(mensaje_clone)
                } else {
                    addr_self.do_send(EstacionDesconectada{ estacion_id: prox_id, mensaje: mensaje_clone });
                    None
                }
            })
                .map(|maybe_msg, _act: &mut Estacion, ctx: &mut Context<Estacion>| {
                    if let Some(mensaje) = maybe_msg {
                        // cuando la reconexión haya registrado la conexión (AgregarEstacion),
                        // recibiremos Reenviar y volveremos a intentar enviar.
                        ctx.address().do_send(Reenviar(String::from_utf8_lossy(&mensaje).to_string()));
                    }
                }),
        );


        let prox_id = if self.siguiente_estacion + 1 >= self.estaciones_cercanas.len() {
            0
        } else {
            self.siguiente_estacion + 1
        };
        self.siguiente_estacion = prox_id;
        let proxima = self.todas_las_estaciones.get(&prox_id).unwrap().clone();
        let addr_self = ctx.address();
        let self_id = self.id;
        let mensaje_clone = msg.mensaje.clone();

        // Intentar reconectar en background; si tiene éxito, pedir reenvío (Reenviar)
        ctx.spawn(
            actix::fut::wrap_future(async move {
                if Estacion::intentar_conectar(proxima, addr_self.clone(), self_id, prox_id).await.is_ok() {
                    Some(mensaje_clone)
                } else {
                    None
                }
            })
                .map(|maybe_msg, _act: &mut Estacion, ctx: &mut Context<Estacion>| {
                    if let Some(mensaje) = maybe_msg {
                        // cuando la reconexión haya registrado la conexión (AgregarEstacion),
                        // recibiremos Reenviar y volveremos a intentar enviar.
                        ctx.address().do_send(Reenviar(String::from_utf8_lossy(&mensaje).to_string()));
                    }
                }),
        );
        
    }
}