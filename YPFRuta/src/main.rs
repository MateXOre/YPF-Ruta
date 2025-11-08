use actix::prelude::*;
use actix::io::{FramedWrite, WriteHandler};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};
use futures::StreamExt;

// ===== Mensaje para enviar texto a un peer =====
#[derive(Message)]
#[rtype(result = "()")]
struct SendPeerMsg(String);

// ===== Actor de conexión (cliente hacia otro servidor) =====
struct PeerConnection {
    peer_name: String,
    writer: FramedWrite<String, tokio::net::tcp::OwnedWriteHalf, LinesCodec>,
}

impl Actor for PeerConnection {
    type Context = Context<Self>;
}

impl WriteHandler<LinesCodecError> for PeerConnection {}

impl Handler<SendPeerMsg> for PeerConnection {
    type Result = ();

    fn handle(&mut self, msg: SendPeerMsg, _ctx: &mut Context<Self>) {
        self.writer.write(msg.0);
    }
}

impl StreamHandler<Result<String, LinesCodecError>> for PeerConnection {
    fn handle(&mut self, msg: Result<String, LinesCodecError>, ctx: &mut Context<Self>) {
        match msg {
            Ok(line) => println!("📨 [{}] recibió: {}", self.peer_name, line),
            Err(e) => {
                println!("⚠️  Error con {}: {:?}", self.peer_name, e);
                ctx.stop();
            }
        }
    }
}

// ===== Actor del servidor principal =====
struct ServerActor {
    name: String,
    port: u16,
    peers: Vec<Addr<PeerConnection>>,
}

impl Actor for ServerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Escucha conexiones entrantes
        let port = self.port;
        let name = self.name.clone();
        println!("🚀 [{}] escuchando en 127.0.0.1:{}", name, port);

        ctx.spawn(
            async move {
                let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
                loop {
                    let (stream, addr) = listener.accept().await.unwrap();
                    println!("🔗 [{}] conexión entrante desde {:?}", name, addr);

                    let (r, w) = stream.into_split();
                    let writer = FramedWrite::new(w, LinesCodec::new(), Arbiter::current());
                    let conn = PeerConnection {
                        peer_name: format!("{}(incoming)", name),
                        writer,
                    }
                    .start();
                    let reader = FramedRead::new(r, LinesCodec::new()).map(|res| res);
                    PeerConnection::add_stream(reader, &conn);
                }
            }
            .into_actor(self),
        );
    }
}

impl ServerActor {
    async fn connect_to_peer(&mut self, addr: &str, ctx: &mut Context<Self>) {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                let (r, w) = stream.into_split();
                let writer = FramedWrite::new(w, LinesCodec::new(), Arbiter::current());
                let conn = PeerConnection {
                    peer_name: addr.to_string(),
                    writer,
                }
                .start();

                let reader = FramedRead::new(r, LinesCodec::new()).map(|res| res);
                PeerConnection::add_stream(reader, &conn);
                println!("🤝 [{}] conectado a {}", self.name, addr);
                self.peers.push(conn);
            }
            Err(e) => {
                println!("❌ [{}] no pudo conectar a {}: {}", self.name, addr, e);
            }
        }
    }
}

impl Handler<SendPeerMsg> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: SendPeerMsg, _ctx: &mut Context<Self>) {
        for peer in &self.peers {
            peer.do_send(SendPeerMsg(format!("{} dice: {}", self.name, msg.0)));
        }
    }
}

// ======== Punto de entrada ========
#[actix::main]
async fn main() {
    // Creamos tres servidores
    let mut server1 = ServerActor {
        name: "Servidor1".into(),
        port: 9001,
        peers: vec![],
    }
    .start();

    let mut server2 = ServerActor {
        name: "Servidor2".into(),
        port: 9002,
        peers: vec![],
    }
    .start();

    let mut server3 = ServerActor {
        name: "Servidor3".into(),
        port: 9003,
        peers: vec![],
    }
    .start();

    // Esperamos un poco para que los sockets estén listos
    actix_rt::time::sleep(std::time::Duration::from_millis(300)).await;

    // Conectamos entre sí
    server1
        .send(SendPeerMsg("Hola desde 1".into()))
        .await
        .ok();
    server2
        .send(SendPeerMsg("Hola desde 2".into()))
        .await
        .ok();
    server3
        .send(SendPeerMsg("Hola desde 3".into()))
        .await
        .ok();
}