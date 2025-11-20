use actix::{Handler, Context};
use actix::prelude::*;
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;

impl Handler<AgregarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: AgregarEstacion, ctx: &mut Context<Self>) {
        println!("[{}] agregando estación conectada desde {}", self.id, msg.estacion_id);

        self.estaciones_cercanas.push(msg.estacion);


        // Si es la última estación (id = total_estaciones - 1) y tiene su conexión lista, iniciar la ronda
        let es_ultima = self.id == self.total_estaciones - 1;

        if es_ultima && !self.primer_anillo_realizado{
            self.primer_anillo_realizado = true;
            println!("[{}] Soy la última estación, iniciando ronda de mensajes", self.id);
            let siguiente_estacion = self.siguiente_estacion;
            let id_estacion = self.id;
            let addr_self = ctx.address();
            actix_rt::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                let mensaje = Eleccion {
                    aspirantes_ids: vec![id_estacion],
                };
                if let Some(siguiente) = siguiente_estacion {
                    let ids_str: Vec<String> = mensaje.aspirantes_ids.iter().map(|id| id.to_string()).collect();
                    let mensaje_serializado = format!("ANILLO:{}", ids_str.join(","));
                    addr_self.do_send(Reenviar(mensaje_serializado));
                    println!("[{}] enviando mensaje inicial (via Reenviar) a siguiente estación en {}", id_estacion, siguiente);
                }
            });
        }
    }
}