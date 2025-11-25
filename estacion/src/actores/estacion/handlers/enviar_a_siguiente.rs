use crate::actores::estacion::deserialize_message;
use crate::actores::estacion::MessageType::InformarVentasOffline;
use crate::actores::estacion::{EnviarASiguiente, Estacion};
use crate::actores::estacion_cercana::{CerrarConexion, Enviar};
use actix::{AsyncContext, Handler};

impl Handler<EnviarASiguiente> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: EnviarASiguiente, ctx: &mut actix::Context<Self>) {
        println!(
            "[{}] Estacion {} no responde, eliminando de estaciones cercanas",
            self.id, msg.estacion_cercana_id
        );
        if let Some(estacion_desconectada) = self.estaciones_cercanas.get(&msg.estacion_cercana_id)
        {
            estacion_desconectada.do_send(CerrarConexion {});
        }
        self.estaciones_cercanas.remove(&msg.estacion_cercana_id);
        let siguiente_estacion = (msg.estacion_cercana_id + 1) % (self.todas_las_estaciones.len());
        println!(
            "[{}] Error enviando mensaje de anillo a {}: Conectando con siguiente estacion: {}",
            self.id, msg.estacion_cercana_id, siguiente_estacion
        );

        if self.lider_actual == Some(msg.estacion_cercana_id) {
            let mensaje = match deserialize_message(&msg.msg) {
                Ok(mensaje) => mensaje,
                Err(e) => {
                    println!("[{}] Error deserializando mensaje: {}", self.id, e);
                    return;
                }
            };
            if let InformarVentasOffline(mensaje) = mensaje {
                println!(
                    "[{}] El mensaje era para el lider actual {}, no lo reenvio para evitar loops",
                    self.id, msg.estacion_cercana_id
                );
                let ventas_acumuladas = mensaje.ventas;
                self.ventas_por_informar = self.agregar_ventas_acumuladas(ventas_acumuladas);
                self.guardar_ventas_sin_informar();
                return;
            }
        }
        if let Some(siguiente_addr) = self.estaciones_cercanas.get(&siguiente_estacion) {
            siguiente_addr.do_send(Enviar { bytes: msg.msg });
        } else {
            println!("[{}] No se encontró la estacion cercana para la siguiente estación {}, intentando reconectar, ", self.id, siguiente_estacion);
            if let Some(_sig_addr) = self.todas_las_estaciones.get(&siguiente_estacion) {
                ctx.address()
                    .do_send(crate::actores::estacion::messages::EstacionDesconectada {
                        estacion_id: siguiente_estacion,
                        mensaje: msg.msg.clone(),
                    });
            } else {
                println!(
                    "[{}] No se encontró la dirección de la siguiente estación {}",
                    self.id, siguiente_estacion
                );
            }
        }
    }
}
