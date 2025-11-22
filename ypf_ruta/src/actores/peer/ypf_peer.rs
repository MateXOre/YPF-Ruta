use actix::ActorFutureExt;
use crate::actores::peer::messages::ProcesarMensaje;
use crate::actores::ypf::messages::{NuevoLider, PeerDesconectado};
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Actor, Addr, AsyncContext, Context, WrapFuture};
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver};

const PING_INTERVAL_SECS: u64 = 30;

pub struct YpfPeer {
    pub peer_id: usize,
    pub local_id: usize,
    pub cola_envio: Option<UnboundedSender<Vec<u8>>>,
    pub reader: Option<OwnedReadHalf>,
    pub ypf_local_addr: Addr<YpfRuta>,
    pub(crate) last_ping: Instant,
    pub(crate) last_pong: Instant,
    last_ping_sent: Instant,
}

async fn empezar_conexion(ip_addr: SocketAddr, id_local: usize) -> Option<TcpStream> {
    match TcpStream::connect(ip_addr).await {
        Ok(mut s) => {
            s.write_all(format!("ID_LOCAL:{}\n", id_local).as_bytes())
                .await
                .unwrap();
            Some(s)
        }
        Err(_) => None
    }
}

impl YpfPeer {
    fn default(
        id_peer: usize,
        id_local: usize,
        ypf_local_addr: Addr<YpfRuta>
    ) -> Self {
        YpfPeer {
            peer_id: id_peer,
            local_id: id_local,
            cola_envio: None,
            reader: None,
            ypf_local_addr,
            last_ping: Instant::now(),
            last_pong: Instant::now(),
            last_ping_sent: Instant::now(),
        }
    }

    pub async fn new(
        peer_id: usize,
        local_id: usize,
        socket_tcp: Option<TcpStream>,
        addr: Option<SocketAddr>,
        local_addr: Addr<YpfRuta>,
    ) -> Self {
        
        let socket = match Self::obtener_socket(socket_tcp, addr, local_id, peer_id).await {
            Ok(s) => s,
            Err(msg) => {
                eprintln!("{}", msg);
                return YpfPeer::default(peer_id, local_id, local_addr);
            }
        };

        let (reader, writer) = socket.into_split();
        let (tx, rx) = unbounded_channel::<Vec<u8>>();

        let local_clone = local_addr.clone();
        
        // Task que posee el writer y serializa las escrituras
        Self::escribir_a_socket(rx, writer, peer_id, local_clone);

        YpfPeer {
            peer_id,
            local_id,
            cola_envio: Some(tx),
            reader: Some(reader),
            ypf_local_addr: local_addr,
            last_ping: Instant::now(),
            last_pong: Instant::now(),
            last_ping_sent: Instant::now(),
        }
    }

    async fn obtener_socket(socket_tcp: Option<TcpStream>, addr: Option<SocketAddr>, local_id: usize, peer_id: usize) -> Result<TcpStream, String> {
        if let Some(s) = socket_tcp {
            Ok(s)
        } else if let Some(a) = addr {
            match TcpStream::connect(a).await {
                Ok(_) => {
                    println!("Conectado exitosamente al YpfPeer {} en {}", peer_id, a);
                    match empezar_conexion(a, local_id).await {
                        Some(sock) => Ok(sock),
                        None => Err(format!(
                            "No se pudo iniciar la conexión con el YpfPeer {} después de conectar.",
                            peer_id
                        )),
                    }
                }
                Err(e) => Err(format!("No se pudo conectar al YpfPeer {}: {}", peer_id, e)),
            }
        } else {
            Err(format!(
                "YpfPeer {}: No se proporcionó socket ni dirección para la conexión.",
                peer_id
            ))
        }
    }

    pub(crate) fn escribir_a_socket(mut rx:  UnboundedReceiver<Vec<u8>>, mut writer: tokio::net::tcp::OwnedWriteHalf, peer_id: usize, local_clone: Addr<YpfRuta>) {
        tokio::spawn(async move {
            println!("YpfPeer {}: Iniciando tarea de escritura del socket...", peer_id);
            
            while let Some(buf) = rx.recv().await {
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                }
            }

            println!("YpfPeer {}: Tarea de escritura del socket finalizada.", peer_id);
            local_clone.do_send(PeerDesconectado { id: peer_id });
        });
    }
    
    pub fn start_ping_loop(&mut self, ctx: &mut Context<Self>) {
        let socket = match self.cola_envio.as_ref() {
            Some(s) => s.clone(),
            None => {
                eprintln!(
                    "YpfPeer {}: No se puede iniciar el ping loop sin un socket válido.",
                    self.peer_id
                );
                return;
            }
        };

        println!("YpfPeer {}: Iniciando ping loop...", self.peer_id);
        ctx.run_interval(Duration::from_secs(PING_INTERVAL_SECS), move |act, _ctx| {
            act.last_ping_sent = Instant::now();
            if let Err(e) = socket.send(b"0\n".to_vec()) {
                eprintln!("YpfPeer {}: Error enviando PING: {}", act.peer_id, e);
            } else {
                println!("YpfPeer {}: PING enviado.", act.peer_id);
            }
        });
    }

    pub async fn read_from_socket(mut reader: OwnedReadHalf, self_addr: Addr<YpfPeer>, id: usize, local_addr: Addr<YpfRuta>) {
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => {
                        println!("YpfPeer {}: Conexión cerrada por el peer remoto.", id);
                        local_addr.do_send(PeerDesconectado { id });
                        break;
                    }
                    Ok(bytes) => {
                        self_addr.do_send(ProcesarMensaje {
                            bytes: buf[..bytes].to_vec(),
                        });
                    }
                    Err(e) => {
                        eprintln!("YpfPeer {}: Error leyendo del socket: {}", id, e);

                        local_addr.do_send(PeerDesconectado { id });
                        break;

                    }
                }

            }
        });
    }
}

impl Actor for YpfPeer {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("YpfPeer {} iniciado.", self.peer_id);
        self.start_ping_loop(ctx);
        if let Some(reader) = self.reader.take() {
            let self_addr = ctx.address();
            let peer_id = self.peer_id;
            let local_addr = self.ypf_local_addr.clone();
            tokio::spawn(async move {
                println!("YpfPeer {}: Iniciando tarea de lectura del socket...", peer_id);
                YpfPeer::read_from_socket(reader, self_addr, peer_id, local_addr).await;
            });

        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("YpfPeer {} detenido.", self.peer_id);
    }
}
