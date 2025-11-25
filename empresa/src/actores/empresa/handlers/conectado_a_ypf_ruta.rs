use actix::{AsyncContext, Context, Handler};
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use crate::actores::empresa::Empresa;
use crate::actores::empresa::messages::{ConectadoAypfRuta, ResponderConsola};

impl Handler<ConectadoAypfRuta> for Empresa {
    type Result = ();
    fn handle(&mut self, msg: ConectadoAypfRuta, ctx: &mut Context<Self>) {
        println!("[Empresa {}] conectado a ypf ruta", self.id);
        self.ypf_ruta_addr = Some(msg.addr);
        let estacion_address = ctx.address();

        // Leer de la consola en una tarea asíncrona
        actix_rt::spawn(async move {
            let stdin = BufReader::new(io::stdin());
            let mut lines = stdin.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                // Enviar mensaje al actor
                estacion_address.do_send(ResponderConsola { linea: line });
            }
        });
    }
}