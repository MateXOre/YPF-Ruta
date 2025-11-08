use actix::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

// ===== Mensajes =====
#[derive(Message)]
#[rtype(result = "()")]
struct SendPeerMsg(String);

#[derive(Message)]
#[rtype(result = "()")]
struct ReceivedLine(String);

#[derive(Message)]
#[rtype(result = "()")]
struct ConnectPeer {
    addr: String,
}

#[derive(Message)]
#[rtype(result = "()")]
struct AddPeer {
    peer: Addr<PeerConnection>,
}

// ===== Actor de conexión (cliente hacia otro servidor) =====
struct PeerConnection {
    peer_name: String,
    tx: tokio::sync::mpsc::Sender<String>, // para enviar líneas al writer task
}

impl Actor for PeerConnection {
    type Context = Context<Self>;
}

impl Handler<SendPeerMsg> for PeerConnection {
    type Result = ();

    fn handle(&mut self, msg: SendPeerMsg, _ctx: &mut Context<Self>) {
        let _ = self.tx.try_send(msg.0);
    }
}

impl Handler<ReceivedLine> for PeerConnection {
    type Result = ();

    fn handle(&mut self, msg: ReceivedLine, _ctx: &mut Context<Self>) {
        println!("📨 [{}] recibió: {}", self.peer_name, msg.0);
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
        let port = self.port;
        let name = self.name.clone();
        println!("🚀 [{}] escuchando en 127.0.0.1:{}", name, port);

        let addr_self = ctx.address();

        // correr listener en background
        actix_rt::spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        println!("🔗 [{}] conexión entrante desde {:?}", name, peer_addr);
                        // crear actor y tareas de lectura/escritura
                        if let Err(e) = handle_stream_incoming(stream, name.clone(), addr_self.clone()).await
                        {
                            eprintln!("⚠️  error manejando conexión entrante: {:?}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  accept error: {:?}", e);
                    }
                }
            }
        });
    }
}

impl ServerActor {
    async fn connect_and_register(
        target: String,
        actor_addr: Addr<ServerActor>,
        my_name: String,
    ) {
        match TcpStream::connect(&target).await {
            Ok(stream) => {
                println!("🤝 [{}] conectado a {}", my_name, target);
                if let Err(e) = handle_stream_outgoing(stream, target.clone(), actor_addr.clone()).await {
                    eprintln!("❌ [{}] error al manejar conexión con {}: {:?}", my_name, target, e);
                }
            }
            Err(e) => {
                eprintln!("❌ [{}] no pudo conectar a {}: {}", my_name, target, e);
            }
        }
    }
}

impl Handler<ConnectPeer> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: ConnectPeer, _ctx: &mut Context<Self>) {
        let target = msg.addr.clone();
        let my_name = self.name.clone();
        let actor_addr = _ctx.address();
        // spawn async connect (no block actor)
        actix_rt::spawn(async move {
            ServerActor::connect_and_register(target, actor_addr, my_name).await;
        });
    }
}

impl Handler<AddPeer> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: AddPeer, _ctx: &mut Context<Self>) {
        // indicador simple: mostrar si el Addr está conectado (booleano)
        println!("🔌 [{}] agregando peer (connected: {})", self.name, msg.peer.connected());
        self.peers.push(msg.peer);
    }
}

impl Handler<SendPeerMsg> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: SendPeerMsg, _ctx: &mut Context<Self>) {
        for peer in &self.peers {
            let _ = peer.do_send(SendPeerMsg(format!("{} dice: {}", self.name, msg.0)));
        }
    }
}

// ===== funciones auxiliares para manejo de streams =====
async fn handle_stream_incoming(
    stream: TcpStream,
    peer_name: String,
    server_addr: Addr<ServerActor>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (r, mut w) = stream.into_split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // crear actor
    let conn = PeerConnection {
        peer_name: format!("{}(incoming)", peer_name),
        tx: tx.clone(),
    }
    .start();

    // registrar en el servidor
    server_addr.do_send(AddPeer { peer: conn.clone() });

    // escritura: consume rx y escribe al socket
    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if w.write_all(format!("{}\n", line).as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // lectura: usar FramedRead y reenviar líneas al actor
    let mut reader = FramedRead::new(r, LinesCodec::new());
    while let Some(res) = reader.next().await {
        match res {
            Ok(line) => conn.do_send(ReceivedLine(line)),
            Err(e) => {
                eprintln!("⚠️  error lectura entrante en {}: {:?}", peer_name, e);
                break;
            }
        }
    }

    Ok(())
}

async fn handle_stream_outgoing(
    stream: TcpStream,
    peer_name: String,
    server_addr: Addr<ServerActor>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (r, mut w) = stream.into_split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    let conn = PeerConnection {
        peer_name: peer_name.clone(),
        tx: tx.clone(),
    }
    .start();

    // registrar en el servidor
    server_addr.do_send(AddPeer { peer: conn.clone() });

    // escritura
    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if w.write_all(format!("{}\n", line).as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // lectura
    let mut reader = FramedRead::new(r, LinesCodec::new());
    while let Some(res) = reader.next().await {
        match res {
            Ok(line) => conn.do_send(ReceivedLine(line)),
            Err(e) => {
                eprintln!("⚠️  error lectura saliente en {}: {:?}", peer_name, e);
                break;
            }
        }
    }

    Ok(())
}

// ======== Punto de entrada ========
#[actix::main]
async fn main() {
    // crear servidores (actores)
    let server1 = ServerActor {
        name: "Servidor1".into(),
        port: 9001,
        peers: vec![],
    }
    .start();

    let server2 = ServerActor {
        name: "Servidor2".into(),
        port: 9002,
        peers: vec![],
    }
    .start();

    let server3 = ServerActor {
        name: "Servidor3".into(),
        port: 9003,
        peers: vec![],
    }
    .start();

    // esperar a que los listeners estén up
    actix_rt::time::sleep(std::time::Duration::from_millis(300)).await;

    // conectarlos entre sí (ejemplo simple)
    server1.do_send(ConnectPeer {
        addr: "127.0.0.1:9002".into(),
    });
    server2.do_send(ConnectPeer {
        addr: "127.0.0.1:9003".into(),
    });
    server3.do_send(ConnectPeer {
        addr: "127.0.0.1:9001".into(),
    });

    // esperar conexiones
    actix_rt::time::sleep(std::time::Duration::from_millis(500)).await;

    // enviar mensajes que se propagan a peers conocidos
    server1.do_send(SendPeerMsg("Hola desde 1".into()));
    server2.do_send(SendPeerMsg("Hola desde 2".into()));
    server3.do_send(SendPeerMsg("Hola desde 3".into()));

    // mantener el proceso vivo un rato para ver interacción
    actix_rt::time::sleep(std::time::Duration::from_secs(5)).await;
}