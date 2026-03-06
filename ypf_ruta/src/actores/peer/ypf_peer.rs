use crate::actores::peer::messages::ProcesarMensaje;
use crate::actores::ypf::messages::PeerDesconectado;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Actor, Addr, AsyncContext, Context};
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use util::{log_debug, log_error, log_info, log_warning};

const PING_INTERVAL_SECS: u64 = 30;

pub struct YpfPeer {
    pub peer_id: usize,
    pub cola_envio: Option<UnboundedSender<Vec<u8>>>,
    pub reader: Option<OwnedReadHalf>,
    pub ypf_local_addr: Addr<YpfRuta>,
    pub(crate) last_ping: Instant,
    pub(crate) last_pong: Instant,
    last_ping_sent: Instant,
    pub logger: Sender<Vec<u8>>,
}

async fn empezar_conexion(ip_addr: SocketAddr, id_local: usize) -> Option<TcpStream> {
    match TcpStream::connect(ip_addr).await {
        Ok(mut s) => {
            s.write_all(format!("ID_LOCAL:{}\n", id_local).as_bytes())
                .await
                .unwrap();
            Some(s)
        }
        Err(_) => None,
    }
}

impl YpfPeer {
    fn default(id_peer: usize, ypf_local_addr: Addr<YpfRuta>, logger: Sender<Vec<u8>>) -> Self {
        YpfPeer {
            peer_id: id_peer,
            cola_envio: None,
            reader: None,
            ypf_local_addr,
            last_ping: Instant::now(),
            last_pong: Instant::now(),
            last_ping_sent: Instant::now(),
            logger,
        }
    }

    pub async fn new(
        peer_id: usize,
        local_id: usize,
        socket_tcp: Option<TcpStream>,
        addr: Option<SocketAddr>,
        local_addr: Addr<YpfRuta>,
        logger: Sender<Vec<u8>>,
    ) -> Self {
        let socket = match Self::obtener_socket(socket_tcp, addr, local_id, peer_id).await {
            Ok(s) => s,
            Err(msg) => {
                log_error!(
                    logger,
                    "YpfPeer {}: Error al obtener socket - {}",
                    peer_id,
                    msg
                );
                return YpfPeer::default(peer_id, local_addr, logger);
            }
        };

        let (reader, writer) = socket.into_split();
        let (tx, rx) = unbounded_channel::<Vec<u8>>();

        let local_clone = local_addr.clone();
        Self::escribir_a_socket(rx, writer, peer_id, local_clone, logger.clone());

        YpfPeer {
            peer_id,
            cola_envio: Some(tx),
            reader: Some(reader),
            ypf_local_addr: local_addr,
            last_ping: Instant::now(),
            last_pong: Instant::now(),
            last_ping_sent: Instant::now(),
            logger,
        }
    }

    async fn obtener_socket(
        socket_tcp: Option<TcpStream>,
        addr: Option<SocketAddr>,
        local_id: usize,
        peer_id: usize,
    ) -> Result<TcpStream, String> {
        if let Some(s) = socket_tcp {
            Ok(s)
        } else if let Some(a) = addr {
            match TcpStream::connect(a).await {
                Ok(_) => match empezar_conexion(a, local_id).await {
                    Some(sock) => Ok(sock),
                    None => Err(format!(
                        "No se pudo iniciar la conexión con el YpfPeer {} después de conectar.",
                        peer_id
                    )),
                },
                Err(e) => Err(format!("No se pudo conectar al YpfPeer {}: {}", peer_id, e)),
            }
        } else {
            Err(format!(
                "YpfPeer {}: No se proporcionó socket ni dirección para la conexión.",
                peer_id
            ))
        }
    }

    pub(crate) fn escribir_a_socket(
        mut rx: UnboundedReceiver<Vec<u8>>,
        mut writer: tokio::net::tcp::OwnedWriteHalf,
        peer_id: usize,
        local_clone: Addr<YpfRuta>,
        logger: Sender<Vec<u8>>,
    ) {
        tokio::spawn(async move {
            log_debug!(
                logger,
                "YpfPeer {}: Iniciando tarea de escritura del socket...",
                peer_id
            );

            while let Some(buf) = rx.recv().await {
                if let Err(e) = writer.write_all(&buf).await {
                    log_error!(logger, "Error escribiendo en el socket: {}", e);
                    break;
                }
            }

            log_info!(
                logger,
                "YpfPeer {}: Tarea de escritura del socket finalizada.",
                peer_id
            );
            local_clone.do_send(PeerDesconectado { id: peer_id });
        });
    }

    pub fn start_ping_loop(&mut self, ctx: &mut Context<Self>) {
        let socket = match self.cola_envio.as_ref() {
            Some(s) => s.clone(),
            None => {
                log_warning!(
                    self.logger,
                    "YpfPeer {}: No se puede iniciar el ping loop sin un socket válido.",
                    self.peer_id
                );
                return;
            }
        };

        log_debug!(
            self.logger,
            "YpfPeer {}: Iniciando ping loop...",
            self.peer_id
        );
        ctx.run_interval(Duration::from_secs(PING_INTERVAL_SECS), move |act, _ctx| {
            act.last_ping_sent = Instant::now();
            if let Err(e) = socket.send(b"0\n".to_vec()) {
                log_error!(
                    act.logger,
                    "YpfPeer {}: Error enviando PING: {}",
                    act.peer_id,
                    e
                );
            } else {
                log_debug!(act.logger, "YpfPeer {}: PING enviado.", act.peer_id);
            }
        });
    }

    pub async fn read_from_socket(
        reader: OwnedReadHalf,
        self_addr: Addr<YpfPeer>,
        id: usize,
        local_addr: Addr<YpfRuta>,
        logger: Sender<Vec<u8>>,
    ) {
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut buf_reader = tokio::io::BufReader::new(reader);

            loop {
                let mut line = Vec::new();
                match buf_reader.read_until(b'\n', &mut line).await {
                    Ok(0) => {
                        log_warning!(
                            logger,
                            "YpfPeer {}: Conexión cerrada por el peer remoto.",
                            id
                        );
                        local_addr.do_send(PeerDesconectado { id });
                        break;
                    }
                    Ok(_) => {
                        self_addr.do_send(ProcesarMensaje { bytes: line });
                    }
                    Err(e) => {
                        log_error!(logger, "YpfPeer {}: Error leyendo del socket: {}", id, e);
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
        let logger = self.logger.clone();
        log_info!(logger, "YpfPeer {}: Iniciado correctamente.", self.peer_id);
        if self.cola_envio.is_some() {
            self.ypf_local_addr
                .do_send(crate::actores::ypf::messages::SocketListo {
                    peer_id: self.peer_id,
                });
        }

        self.start_ping_loop(ctx);
        if let Some(reader) = self.reader.take() {
            let self_addr = ctx.address();
            let peer_id = self.peer_id;
            let local_addr = self.ypf_local_addr.clone();
            tokio::spawn(async move {
                log_debug!(
                    logger,
                    "YpfPeer {}: Iniciando tarea de lectura del socket...",
                    peer_id
                );
                YpfPeer::read_from_socket(reader, self_addr, peer_id, local_addr, logger).await;
            });
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log_warning!(self.logger, "YpfPeer {}: Deteniéndose...", self.peer_id);
    }
}
