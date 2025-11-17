use actix::{Context, Handler};
use tokio::io::AsyncWriteExt;

use crate::actores::surtidor::{actor::Surtidor, messages::ResultadoVenta};

impl Handler<ResultadoVenta> for Surtidor {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVenta, _ctx: &mut Context<Self>) {
        if let Some(mut wu) = self.writer.take() {
            actix_rt::spawn(async move {
                let respuesta = if msg.exito {
                    format!("Venta exitosa. ID de venta: {}\n", msg.id_venta)
                } else {
                    "Venta fallida.\n".to_string()
                };
                if let Err(e) = wu.write_all(respuesta.as_bytes()).await {
                    println!("Error al escribir al cliente: {:?}", e);
                } else {
                    println!("Respuesta enviada al cliente");
                }
            });
        } else {
            println!("No hay writer disponible (take devolvió None)");
        }
    }
    
}
