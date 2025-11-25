use crate::actores::estacion::SurtidorLibre;
use crate::actores::surtidor::messages::*;
use crate::actores::surtidor::surtidor::Surtidor;
use actix::prelude::*;
use actix::{Context, Handler};

impl Handler<Detenerme> for Surtidor {
    type Result = ();

    fn handle(&mut self, _msg: Detenerme, ctx: &mut Context<Self>) {
        println!("[{}] Deteniendo Surtidor {}", self.estacion_id, self.id);
        self.estacion.do_send(SurtidorLibre {
            surtidor_id: self.id,
        });

        //Señal para que la task de escritura termine
        let _ = self.writer_tx.send(Vec::new());

        //Cerrar reader si aún está activo
        if let Some(reader) = self.reader.take() {
            drop(reader); // cerrar TcpStream
        }

        //Detener el actor
        ctx.stop();
    }
}
