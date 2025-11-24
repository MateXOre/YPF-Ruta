use crate::actores::peer::messages::GuardarSocket;
use crate::actores::peer::ypf_peer::YpfPeer;
use actix::{AsyncContext, Context, Handler};
use tokio::sync::mpsc::unbounded_channel;

impl Handler<GuardarSocket> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: GuardarSocket, ctx: &mut Context<Self>) -> Self::Result {
        println!("YpfPeer {}: Guardando socket entrante...", self.peer_id);

        let (reader, writer) = msg.0.into_split();
        let (tx, rx) = unbounded_channel::<Vec<u8>>();

        self.cola_envio = Some(tx);
        let local_clone = self.ypf_local_addr.clone();
        let peer_id = self.peer_id;

        // Task que posee el writer y serializa las escrituras
        Self::escribir_a_socket(rx, writer, peer_id, local_clone);

        self.start_ping_loop(ctx);

        let self_addr = ctx.address();
        let peer_id = self.peer_id;
        let local_addr = self.ypf_local_addr.clone();
        tokio::spawn(async move {
            println!(
                "YpfPeer {}: Iniciando tarea de lectura del socket...",
                peer_id
            );
            YpfPeer::read_from_socket(reader, self_addr, peer_id, local_addr).await;
        });

        println!("Socket guardado exitosamente para YpfPeer {}", self.peer_id);

        // Notificar al YpfRuta que el socket está listo
        self.ypf_local_addr
            .do_send(crate::actores::ypf::messages::SocketListo {
                peer_id: self.peer_id,
            });
    }
}
