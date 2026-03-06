use crate::actores::surtidor::messages::Detenerme;
use crate::actores::surtidor::{messages::ResultadoVenta, surtidor::Surtidor};
use actix::{AsyncContext, Context, Handler};
use util::{log_error, log_info, log_warning};

impl Handler<ResultadoVenta> for Surtidor {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVenta, ctx: &mut Context<Self>) {
        let respuesta = if msg.exito {
            log_info!(
                self.logger,
                "[{}] ({}) Venta exitosa",
                self.estacion_id,
                self.id
            );
            "Venta exitosa.\n".to_string()
        } else {
            log_warning!(
                self.logger,
                "[{}] ({}) Venta Rechazada",
                self.estacion_id,
                self.id
            );
            "Venta fallida.\n".to_string()
        };

        if let Err(e) = self.writer_tx.send(respuesta.into_bytes()) {
            log_error!(self.logger, "Error al enviar respuesta al writer: {:?}", e);
        } else {
            log_info!(self.logger, "Respuesta enviada al cliente");
            ctx.address().do_send(Detenerme);
        }
    }
}
