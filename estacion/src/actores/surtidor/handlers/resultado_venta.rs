use crate::actores::surtidor::messages::Detenerme;
use crate::actores::surtidor::{messages::ResultadoVenta, surtidor::Surtidor};
use actix::{AsyncContext, Context, Handler};

impl Handler<ResultadoVenta> for Surtidor {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVenta, ctx: &mut Context<Self>) {
        let respuesta = if msg.exito {
            println!("Venta exitosa");
            "Venta exitosa.\n".to_string()
        } else {
            println!("Venta Rechazada");
            "Venta fallida.\n".to_string()
        };

        if let Err(e) = self.writer_tx.send(respuesta.into_bytes()) {
            println!("Error al enviar respuesta al writer: {:?}", e);
        } else {
            println!("Respuesta enviada al cliente");
            ctx.address().do_send(Detenerme);
        }
    }
}
