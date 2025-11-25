use crate::actores::estacion::{Eleccion, Estacion, EstacionDesconectada, LiderCaido};
use crate::actores::estacion_cercana::Enviar;
use actix::{AsyncContext, Handler};

impl Handler<LiderCaido> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: LiderCaido, ctx: &mut actix::Context<Self>) -> Self::Result {
        println!(
            "[{}] El líder ha caído, eliminándolo de estaciones cercanas",
            self.id
        );

        if let Some(id_lider) = self.lider_actual {
            if let Some(estacion) = self.estaciones_cercanas.get(&id_lider) {
                estacion.do_send(crate::actores::estacion_cercana::CerrarConexion {});
            }
            self.estaciones_cercanas.remove(&id_lider);
            self.lider_actual = None;
        }

        println!("Guardo Ventas a Informar");

        self.ventas_por_informar
            .entry(self.id)
            .or_default()
            .entry(msg.mensaje.id_surtidor)
            .or_default()
            .push(msg.mensaje.venta);
        self.guardar_ventas_sin_informar();

        println!("[{}] Iniciando elección de nuevo líder", self.id);

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
            actix_rt::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                siguiente.do_send(Enviar {
                    bytes: mensaje.to_bytes(),
                });
                println!(
                    "[{}] Enviando mensaje inicial a estación {}",
                    id, siguiente_estacion
                );
            });
        } else {
            println!(
                "[{}] Estación {} no conectada, reintentando conexión",
                id, siguiente_estacion
            );
            ctx.address().do_send(EstacionDesconectada {
                estacion_id: self.siguiente_estacion,
                mensaje: mensaje.to_bytes(),
            });
        }
    }
}
