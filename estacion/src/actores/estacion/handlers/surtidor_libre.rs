use actix::{AsyncContext, Handler};
use crate::actores::estacion::{deserialize_message, AceptarCliente, Estacion, MessageType, ProcesarMensaje, SurtidorLibre};

impl Handler<SurtidorLibre> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: SurtidorLibre, ctx: &mut Self::Context) -> Self::Result {
        self.surtidores.remove(&msg.surtidor_id);
        if let Some(cliente_en_espera) = self.cola_espera.pop_front() {
            ctx.address().do_send(AceptarCliente {
                stream: cliente_en_espera.stream,
                peer_addr: cliente_en_espera.peer_addr,
            });
        }
    }
}