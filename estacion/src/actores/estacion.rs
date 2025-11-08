use actix::{Actor, Context, Handler, Message};
use actix::prelude::*;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_util::codec::{FramedRead, LinesCodec};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use std::io::Write;

use crate::actores::estacion_cercana::EstacionCercana;


#[derive(Message)]
#[rtype(result = "()")]
pub struct ConectarEstacion(pub String);


#[derive(Message)]
#[rtype(result = "()")]
pub struct RespuestaConexion(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct AgregarEstacion {
    pub peer: Addr<EstacionCercana>,
}


struct Estacion {
    id: String,
    port: u16,
    estaciones_cercanas: Vec<Addr<EstacionCercana>>,
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let port = self.port;
        let id = self.id.clone();
        println!("🚀 [{}] escuchando en 127.0.0.1:{}", id, port);

        let addr_self = ctx.address();

        // correr listener en background
        actix_rt::spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        println!("🔗 [{}] conexión entrante desde {:?}", id, peer_addr);
                        // crear actor y tareas de lectura/escritura
                        if let Err(e) = handle_stream_incoming(stream, id.clone(), addr_self.clone()).await
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

impl Estacion {
    async fn connect_and_register(
        target: String,
        actor_addr: Addr<Estacion>,
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

impl Handler<ConectarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ConectarEstacion, _ctx: &mut Context<Self>) {
        let target = msg.0.clone();
        let my_name = self.id.clone();
        let actor_addr = _ctx.address();
        // spawn async connect (no block actor)
        actix_rt::spawn(async move {
            Estacion::connect_and_register(target, actor_addr, my_name).await;
        });
    }
}

impl Handler<AgregarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: AgregarEstacion, _ctx: &mut Context<Self>) {
        // indicador simple: mostrar si el Addr está conectado (booleano)
        println!("🔌 [{}] agregando estación (connected: {})", self.id, msg.peer.connected());
        self.estaciones_cercanas.push(msg.peer);
    }
}

impl Handler<RespuestaConexion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: RespuestaConexion, _ctx: &mut Context<Self>) {
        for peer in &self.estaciones_cercanas {
            //let _ = peer.do_send(RespuestaConexion(format!("{} dice: {}", self.id, msg.0)));
            println!("📨 [{}] recibí de estación cercana: {:?}", self.id, peer);
        }
    }
}


async fn handle_stream_incoming(
    stream: TcpStream,
    peer_name: String,
    server_addr: Addr<Estacion>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (r, mut w) = stream.into_split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // crear actor
    let conn = EstacionCercana {
        estacion_id: format!("{}(incoming)", peer_name),
        tx: tx.clone(),
    }
    .start();

    // registrar en el servidor
    server_addr.do_send(AgregarEstacion { peer: conn.clone() });

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
            Ok(line) => conn.do_send(crate::actores::estacion_cercana::RespuestaConexion(line)),
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
    server_addr: Addr<Estacion>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (r, mut w) = stream.into_split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    let conn = EstacionCercana {
        estacion_id: format!("{}(outgoing)", peer_name),
        tx: tx.clone(),
    }
    .start();

    // registrar en el servidor
    server_addr.do_send(AgregarEstacion { peer: conn.clone() });

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
            Ok(line) => conn.do_send(crate::actores::estacion_cercana::RespuestaConexion(line)),
            Err(e) => {
                eprintln!("⚠️  error lectura saliente en {}: {:?}", peer_name, e);
                break;
            }
        }
    }

    Ok(())
}