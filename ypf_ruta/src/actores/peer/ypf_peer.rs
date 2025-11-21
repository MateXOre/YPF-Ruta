use crate::actores::peer::messages::ProcesarMensaje;
use crate::actores::ypf::messages::PeerDesconectado;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Actor, Addr, AsyncContext, Context};
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

const PING_INTERVAL_SECS: u64 = 30;
//const TIMEOUT_SECS: u64 = 5;

pub struct YpfPeer {
    pub peer_id: usize,
    pub local_id: usize,
    //pub lider_id: Option<usize>,
    pub cola_envio: Option<UnboundedSender<Vec<u8>>>,
    pub reader: Option<OwnedReadHalf>,
    pub ypf_local_addr: Addr<YpfRuta>,
    pub(crate) last_ping: Instant,
    pub(crate) last_pong: Instant,
    last_ping_sent: Instant,
}

impl YpfPeer {
    pub async fn new(
        peer_id: usize,
        local_id: usize,
        //lider_id: Option<usize>,
        socket: Option<TcpStream>,
        addr: Option<SocketAddr>,
        local_addr: Addr<YpfRuta>,
    ) -> Self {

        let socket_f : TcpStream;

        if let Some(s) = socket {
            socket_f = s;
        } else {
            if let Some(a) = addr {
                match tokio::net::TcpStream::connect(a).await {
                    Ok(mut s) => {
                        println!("Conectado exitosamente al YpfPeer {} en {}", peer_id, a);

                        // enviamos nuestro ID_LOCAL
                        s.write_all(format!("ID_LOCAL:{}\n", local_id).as_bytes())
                            .await
                            .unwrap();
                        socket_f = s;
                    }
                    Err(e) => {
                        eprintln!("No se pudo conectar al YpfPeer {}: {}", peer_id, e);
                        return YpfPeer {
                            peer_id,
                            local_id,
                            //lider_id,
                            cola_envio: None,
                            reader: None,
                            ypf_local_addr: local_addr,
                            last_ping: Instant::now(),
                            last_pong: Instant::now(),
                            last_ping_sent: Instant::now(),
                        };
                    }
                };
            } else {
                eprintln!(
                    "YpfPeer {}: No se proporcionó socket ni dirección para la conexión.",
                    peer_id
                );
                return YpfPeer {
                    peer_id,
                    local_id,
                    //lider_id,
                    cola_envio: None,
                    reader: None,
                    ypf_local_addr: local_addr,
                    last_ping: Instant::now(),
                    last_pong: Instant::now(),
                    last_ping_sent: Instant::now(),
                };
            }
        }

        let (reader, mut writer) = socket_f.into_split();

        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();

        let local_clone = local_addr.clone();
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

            println!("YpfPeer {}: Tarea de escritura del socket finalizada.", peer_id);
            local_clone.do_send(PeerDesconectado { id: peer_id });
        });

        YpfPeer {
            peer_id,
            local_id,
            //lider_id,
            cola_envio: Some(tx),
            reader: Some(reader),
            ypf_local_addr: local_addr,
            last_ping: Instant::now(),
            last_pong: Instant::now(),
            last_ping_sent: Instant::now(),
        }
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
