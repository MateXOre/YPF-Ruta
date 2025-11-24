use crate::actores::surtidor::messages::Detenerme;
use crate::actores::surtidor::{messages::ResultadoVenta, surtidor::Surtidor};
use actix::{AsyncContext, Context, Handler};

impl Handler<ResultadoVenta> for Surtidor {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVenta, ctx: &mut Context<Self>) {
        let respuesta = if msg.exito {
            println!("Venta exitosa");
            format!("Venta exitosa.\n")
        } else {
            println!("Venta Rechazada");
            "Venta fallida.\n".to_string()
        };

        // Enviar la respuesta al cliente mediante el canal
        if let Err(e) = self.writer_tx.send(respuesta.into_bytes()) {
            println!("Error al enviar respuesta al writer: {:?}", e);
            return;
        } else {
            println!("Respuesta enviada al cliente");

            ctx.address().do_send(Detenerme);
        }
    }
}
