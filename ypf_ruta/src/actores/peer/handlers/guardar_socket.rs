use actix::{AsyncContext, Context, Handler};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::unbounded_channel;
use crate::actores::peer::messages::GuardarSocket;
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::PeerDesconectado;

impl Handler<GuardarSocket> for YpfPeer {
    type Result = ();

    fn handle(&mut self, msg: GuardarSocket, ctx: &mut Context<Self>) -> Self::Result {
        println!("YpfPeer {}: Guardando socket entrante...", self.peer_id);
        self.cola_envio.as_mut().map(|tx| {
            tx.send(Vec::from(format!("ID_LOCAL:{}\n", self.local_id).as_bytes())).expect("Fallo al enviar VentaRegistrada a la cola de envio");
            tx.send(Vec::from(format!("ID_LIDER:{}\n", self.peer_id).as_bytes())).expect("Fallo al enviar VentaRegistrada a la cola de envio");
        });

        let (reader, mut writer) = msg.0.into_split();

        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();
        self.cola_envio = Some(tx);
        let local_clone = self.ypf_local_addr.clone();
        let peer_id = self.peer_id;
        // Task que posee el writer y serializa las escrituras
        tokio::spawn(async move {
            println!("YpfPeer {}: Iniciando tarea de escritura del socket...", peer_id);
            while let Some(buf) = rx.recv().await {
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                } else {
                    println!("YpfPeer {}: Mensaje enviado por socket.", peer_id);
                }
            }

            println!("YpfPeer {}: Tarea de escritura del socket finalizada. [linea 34]", peer_id);
            local_clone.do_send(PeerDesconectado { id: peer_id });
        });
       
        self.start_ping_loop(ctx);
        let self_addr = ctx.address();
        let peer_id = self.peer_id;
        let local_addr = self.ypf_local_addr.clone();
        tokio::spawn(async move {
            println!("YpfPeer {}: Iniciando tarea de lectura del socket...", peer_id);
            YpfPeer::read_from_socket(reader, self_addr, peer_id, local_addr).await;
        });

        println!("Socket guardado exitosamente para YpfPeer {}", self.peer_id);
    }
}