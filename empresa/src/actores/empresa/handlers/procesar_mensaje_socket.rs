use crate::actores::empresa::messages::{
    deserialize_respuesta_ypfruta, ProcesarMensajeSocket, RespuestaYpfRuta,
};
use crate::actores::empresa::Empresa;
use actix::{AsyncContext, Context, Handler};

impl Handler<ProcesarMensajeSocket> for Empresa {
    type Result = ();
    fn handle(&mut self, msg: ProcesarMensajeSocket, ctx: &mut Context<Self>) {
        match deserialize_respuesta_ypfruta(&msg.bytes) {
            Ok(respuesta) => match respuesta {
                RespuestaYpfRuta::ConfigurarLimite(m) => ctx.address().do_send(m),
                RespuestaYpfRuta::ConfigurarLimiteGeneral(m) => ctx.address().do_send(m),
                RespuestaYpfRuta::GastosEmpresa(m) => ctx.address().do_send(m),
            },
            Err(e) => eprintln!(
                "[Empresa {}] Error deserializando respuesta: {}",
                self.id, e
            ),
        }
    }
}
