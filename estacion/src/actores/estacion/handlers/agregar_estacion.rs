use actix::{Handler, Context, AsyncContext};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use crate::actores::estacion_cercana::Enviar;

impl Handler<AgregarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: AgregarEstacion, ctx: &mut Context<Self>) {
        println!("[{}] agregando estación conectada desde {}", self.id, msg.estacion_id);

        self.estaciones_cercanas.insert(msg.estacion_id, msg.estacion);
        if msg.estacion_id == self.id + 1 || (self.id == self.total_estaciones -1 && msg.estacion_id == 0) {
            println!("[{}] Conexión con siguiente estación {} establecida", self.id, msg.estacion_id);
            self.siguiente_estacion = msg.estacion_id;
        }

        // Si es la última estación (id = total_estaciones - 1) y tiene su conexión lista, iniciar la ronda
        let es_ultima = self.id == self.total_estaciones - 1;

        if es_ultima && !self.primer_anillo_realizado{
            self.primer_anillo_realizado = true;
            println!("[{}] Soy la última estación, iniciando ronda de mensajes", self.id);
            let id_estacion = self.id;
            let mensaje = Eleccion {
                aspirantes_ids: vec![id_estacion],
            };
            let siguiente_estacion = self.siguiente_estacion;
            let siguiente = if self.estaciones_cercanas.get(&siguiente_estacion).is_some() {
                self.estaciones_cercanas.get(&siguiente_estacion).unwrap().clone()
            } else {
                
                println!("[{}] La siguiente estación {} no está conectada, no se puede iniciar el anillo, reintentando conexion", self.id, siguiente_estacion);
                ctx.address().do_send(EstacionDesconectada {
                    estacion_id: siguiente_estacion,
                    mensaje: mensaje.to_bytes(),
                });
                return;
            };
            
            actix_rt::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                siguiente.do_send(Enviar{bytes: mensaje.to_bytes()});
                println!("[{}] enviando mensaje inicial (via Reenviar) a siguiente estación en {}", id_estacion, siguiente_estacion);
                
            });
        }
    }
}