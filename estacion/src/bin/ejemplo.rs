use actix::prelude::*;
use std::net::SocketAddr;

// === Mensajes ===

#[derive(Message)]
#[rtype(result = "()")]
struct PeerMessage(String);

#[derive(Message)]
#[rtype(result = "()")]
struct NewConnection(SocketAddr);

// === Actor principal ===

struct MainServer {
    peers: Vec<Addr<PeerConnector>>,
}

impl Actor for MainServer {
    type Context = Context<Self>;
}

impl Handler<NewConnection> for MainServer {
    type Result = ();

    fn handle(&mut self, msg: NewConnection, ctx: &mut Self::Context) {
        println!("Nueva conexión desde {:?}", msg.0);
        // Podrías crear un actor ConnectionHandler acá
        // y registrarlo en una lista
    }
}

// === Actor que se conecta con otros servidores ===

struct PeerConnector {
    addr: SocketAddr,
}

impl Actor for PeerConnector {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Conectado con peer {:?}", self.addr);
    }
}

impl Handler<PeerMessage> for PeerConnector {
    type Result = ();

    fn handle(&mut self, msg: PeerMessage, _ctx: &mut Context<Self>) {
        println!("Recibido de peer {:?}: {}", self.addr, msg.0);
        // Aquí podrías escribir al socket o reenviar mensajes
    }
}

// === Punto de entrada ===

#[actix::main]
async fn main() {
    // Supongamos que conocemos dos servidores vecinos
    let peers = vec![
        "127.0.0.1:9001".parse().unwrap(),
        "127.0.0.1:9002".parse().unwrap(),
    ];

    let peer_actors: Vec<_> = peers
        .into_iter()
        .map(|addr| PeerConnector { addr }.start())
        .collect();

    let server = MainServer { peers: peer_actors }.start();

    // Enviar un mensaje a todos los peers
    // for peer in &server.try_send(PeerMessage("ping".into())) {}
}
