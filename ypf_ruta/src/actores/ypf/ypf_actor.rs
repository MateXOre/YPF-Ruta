use crate::actores::estacion::estacion_actor::Estacion;
use crate::actores::estacion::messages::{ResultadoVentas, Solicitud};
use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::ValidarVenta;
use crate::actores::peer::messages::VentaRegistrada;
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::ConexionEntrante;
use crate::actores::ypf::EmpresaConectada;
use actix::ActorFutureExt;
use actix::{Actor, Addr, AsyncContext, WrapFuture};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use util::logs::log_info;

use crate::actores::ypf::io::incoming::handle_stream_incoming;

use util::{
    log_debug, log_error,
    structs::venta::{EstadoVenta, Venta},
};

const DIRECCION_IP: &str = "127.0.0.1";
const OFFSET_ESTACIONES: usize = 10000;
const OFFSET_EMPRESAS: usize = 11000;

pub struct YpfRuta {
    pub(crate) id: usize,
    puerto: usize,

    pub(crate) lider: Option<usize>,
    pub(crate) ypf_peers: HashMap<usize, Addr<YpfPeer>>,
    peer_addrs: HashMap<usize, SocketAddr>,
    pub(crate) en_eleccion: bool,
    pub(crate) respuestas_recibidas: usize,

    pub(crate) gestor_addr: Addr<Gestor>,
    pub(crate) ventas_por_confirmar: VecDeque<(Addr<Estacion>, Solicitud)>,
    pub(crate) logger: Sender<Vec<u8>>,
    pub(crate) empresas_activas: HashMap<usize, Addr<EmpresaConectada>>,
}

impl YpfRuta {
    pub fn new(
        id: usize,
        puerto: usize,
        lider: Option<usize>,
        peers: HashMap<usize, SocketAddr>,
        gestor_addr: Addr<Gestor>,
        logger: Sender<Vec<u8>>
    ) -> Self {
        let ypf_peers = HashMap::new();
        log_info!(logger, "YpfRuta {}: Creando instancia", id);

        let ventas_por_confirmar = VecDeque::new();
        YpfRuta {
            id,
            puerto,
            lider,
            ypf_peers,
            peer_addrs: peers,
            en_eleccion: false,
            respuestas_recibidas: 0,
            gestor_addr,
            ventas_por_confirmar,
            logger,
            empresas_activas: HashMap::new(),
        }
    }

    pub(crate) fn spawn_peer(
        &mut self,
        peer_id: usize,
        socket: Option<tokio::net::TcpStream>,
        addr_opt: Option<std::net::SocketAddr>,
        ctx: &mut actix::Context<YpfRuta>,
    ) {
        let self_id = self.id;
        let self_addr = ctx.address();
        let logger = self.logger.clone();
        let fut = async move {
            YpfPeer::new(
                peer_id,
                self_id,
                socket,
                addr_opt,
                self_addr,
                logger.clone(),
            )
            .await
        };

        let logger_fut = self.logger.clone();
        let fut = fut.into_actor(self).map(move |peer, act, _ctx| {
            let addr = peer.start();
            act.ypf_peers.insert(peer_id, addr);
            log_info!(
                logger_fut,
                "YpfRuta {}: Peer {} registrado desde conexión entrante",
                act.id,
                peer_id
            );
        });

        ctx.spawn(fut);
    }

    fn intentar_conectar_peers(&mut self, ctx: &mut actix::Context<Self>) {
        log_info!(
            self.logger,
            "YpfRuta {}: Intentando conectar a peers...",
            self.id
        );
        let peers = self.peer_addrs.clone();
        let self_id = self.id;

        for (peer_id, addr) in peers {
            let self_addr = ctx.address().clone();
            let logger_clone = self.logger.clone();
            let logger_clone2 = self.logger.clone();

            let fut = async move {
                YpfPeer::new(peer_id, self_id, None, Some(addr), self_addr, logger_clone).await
            };
            let id = peer_id;
            let self_id_clone = self_id;

            let fut = fut.into_actor(self).map(move |peer, act, _ctx| {
                let addr = peer.start();
                act.ypf_peers.insert(id, addr);
                log_info!(
                    logger_clone2,
                    "YpfRuta {}: Peer {} iniciado.",
                    self_id_clone,
                    id
                );
            });

            ctx.spawn(fut);
        }
    }

    fn escuchar_peers(&mut self, ctx: &mut actix::Context<Self>) {
        log_info!(
            self.logger,
            "YpfRuta {}: Escuchando conexiones entrantes de peers...",
            self.id
        );
        let puerto = self.puerto as u16;
        let self_id = self.id;
        let self_addr = ctx.address();
        let logger = self.logger.clone();
        ctx.spawn(
            async move {
                let listener = tokio::net::TcpListener::bind(("127.0.0.1", puerto))
                    .await
                    .unwrap();
                loop {
                    if let Ok((socket, _)) = listener.accept().await {
                        let mut reader = tokio::io::BufReader::new(socket);
                        let mut line = String::new();
                        match tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await {
                            Ok(_) => {
                                log_debug!(
                                    logger,
                                    "YpfRuta {}: Línea recibida del socket entrante: {}",
                                    self_id,
                                    line.trim_end()
                                );
                                // parsear "ID_LOCAL:123\n"
                                if let Some(id_str) = line.strip_prefix("ID_LOCAL:") {
                                    if let Ok(peer_id) = id_str.trim().parse::<usize>() {
                                        log_debug!(
                                            logger,
                                            "YpfRuta {}: Conexión entrante del peer {}",
                                            self_id,
                                            peer_id
                                        );

                                        let socket = reader.into_inner();
                                        self_addr.do_send(ConexionEntrante { peer_id, socket });
                                    } else {
                                        log_error!(
                                            logger,
                                            "YpfRuta {}: ID inválido recibido: {}",
                                            self_id,
                                            line.trim_end()
                                        );
                                    }
                                } else {
                                    log_error!(
                                        logger,
                                        "YpfRuta {}: Formato de mensaje inválido. Se esperaba: ID_LOCAL:<ID>, recibido: {}",
                                        self_id,
                                        line
                                    );
                                }
                            }
                            Err(e) => {
                                log_error!(
                                    logger,
                                    "YpfRuta {}: Error leyendo socket: {}",
                                    self_id,
                                    e
                                );
                            }
                        }
                    }
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        );
    }

    pub fn escuchar_estaciones(&mut self, ctx: &mut actix::Context<Self>) {
        if let Some(lider) = self.lider
            && lider == self.id
        {
            log_debug!(
                self.logger,
                "YpfRuta {}: SOY LIDER, Escuchando conexiones entrantes de estaciones líderes...",
                self.id
            );
        } else {
            return;
        }

        let puerto = (self.puerto + OFFSET_ESTACIONES) as u16;
        let self_id = self.id;
        let self_addr = ctx.address();
        let logger = self.logger.clone();
        ctx.spawn(
            async move {
                let listener =
                    if let Ok(l) = tokio::net::TcpListener::bind((DIRECCION_IP, puerto)).await {
                        l
                    } else {
                        log_error!(
                            logger,
                            "YpfRuta {}: No se pudo crear el listener para conexión de estaciones",
                            self_id
                        );
                        return;
                    };

                log_info!(
                    logger,
                    "YpfRuta {}: Listener de estaciones activo en puerto {}",
                    self_id,
                    puerto
                );

                loop {
                    if let Ok((socket, peer_addr)) = listener.accept().await {
                        log_info!(
                            logger,
                            "YpfRuta {}: Nueva conexión de estación desde {:?}",
                            self_id,
                            peer_addr
                        );
                        let log = logger.clone();
                        match Estacion::new(socket, self_addr.clone(), log).await {
                            Ok(estacion_addr) => estacion_addr.start(),
                            Err(_) => continue,
                        };
                        log_debug!(
                            logger,
                            "YpfRuta {}: Actor de Estacion creado para {:?}",
                            self_id,
                            peer_addr
                        );
                    }
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        );
    }

    fn procesar_ventas(&mut self, ctx: &mut actix::Context<Self>) {
        let logger = self.logger.clone();

        log_debug!(
            logger,
            "YpfRuta {}: Iniciando procesador de cola de ventas...",
            self.id
        );
        ctx.run_interval(std::time::Duration::from_millis(100), move |act, _ctx| {
            if let Some((estacion_addr, ventas)) = act.ventas_por_confirmar.pop_front() {
                log_debug!(
                    logger,
                    "YpfRuta {}: Procesando {} ventas de la cola",
                    act.id,
                    ventas.len()
                );

                let gestor = act.gestor_addr.clone();
                let ypf_id = act.id;
                let ypfs_addr: Vec<Addr<YpfPeer>> = act.ypf_peers.values().cloned().collect();
                let log = logger.clone();

                actix::spawn(async move {
                    let mut ventas_aprobadas = Vec::new();
                    let mut resultados_por_estacion = HashMap::new();
                    for (estacion, ventas_por_surtidor) in ventas {
                        let mut resultado_por_surtidor = HashMap::new();
                        for (surtidor, vec_ventas) in ventas_por_surtidor {
                            let mut resultado_ventas = Vec::new();
                            for venta in vec_ventas {
                                log_debug!(
                                    log,
                                    "YpfRuta {}: Validando venta {} en el gestor",
                                    ypf_id,
                                    venta.id_venta
                                );

                                match gestor.send(ValidarVenta(venta.clone())).await {
                                    Ok(estado) => {
                                        let aprobada = match estado {
                                            EstadoVenta::Confirmada => true,
                                            EstadoVenta::Rechazada => false,
                                            _ => false,
                                        };
                                        log_debug!(
                                            log,
                                            "YpfRuta {}: Venta {} - Resultado: {}",
                                            ypf_id,
                                            venta.id_venta,
                                            aprobada
                                        );
                                        if !venta.offline {
                                            resultado_ventas.push((venta.id_venta, aprobada));
                                        }
                                        let venta_actualizada = Venta {
                                            id_venta: venta.id_venta,
                                            id_tarjeta: venta.id_tarjeta,
                                            id_estacion: venta.id_estacion,
                                            monto: venta.monto,
                                            offline: venta.offline,
                                            estado: estado.clone(),
                                        };
                                        if estado == EstadoVenta::Confirmada
                                            || venta_actualizada.offline
                                        {
                                            ventas_aprobadas.push(venta_actualizada);
                                        }
                                    }
                                    Err(e) => {
                                        log_error!(
                                            log,
                                            "YpfRuta {}: Error validando venta {}: {:?}",
                                            ypf_id,
                                            venta.id_venta,
                                            e
                                        );
                                    }
                                }
                            }
                            if !resultado_ventas.is_empty() {
                                resultado_por_surtidor.insert(surtidor, resultado_ventas);
                            }
                        }

                        if !resultado_por_surtidor.is_empty() {
                            resultados_por_estacion.insert(estacion, resultado_por_surtidor);
                        }
                    }
                    if !resultados_por_estacion.is_empty() {
                        estacion_addr.do_send(ResultadoVentas {
                            ventas: resultados_por_estacion,
                        });
                    }

                    for venta in ventas_aprobadas {
                        log_debug!(
                            log,
                            "YpfRuta {}: Replicando venta {} a {} peers",
                            ypf_id,
                            venta.id_venta,
                            ypfs_addr.len()
                        );
                        for peer in &ypfs_addr {
                            peer.do_send(VentaRegistrada {
                                venta: venta.clone(),
                            });
                        }
                    }
                });
            }
        });
    }

    fn escuchar_empresas(&mut self, ctx: &mut actix::Context<Self>) {
        if let Some(lider) = self.lider
            && lider == self.id
        {
            log_debug!(
                self.logger,
                "YpfRuta {}: SOY LIDER, Escuchando conexiones entrantes de estaciones líderes...",
                self.id
            );
        } else {
            return;
        }


        let puerto = (self.puerto + OFFSET_EMPRESAS) as u16;
        log_info!(
            self.logger,
            "YpfRuta {}: Escuchando conexiones entrantes de empresas en puerto {}",
            self.id,
            puerto
        );
        let self_id = self.id;
        let self_addr = ctx.address().clone();
        let logger = self.logger.clone();
        ctx.spawn(
            async move {
                let listener = tokio::net::TcpListener::bind(("127.0.0.1", puerto))
                    .await
                    .unwrap();
                loop {
                    if let Ok((socket, peer_addr)) = listener.accept().await {
                        log_info!(
                            logger,
                            "YpfRuta {}: Nueva conexión de empresa desde {:?}",
                            self_id,
                            peer_addr
                        );
                        let _ = handle_stream_incoming(socket, self_id, self_addr.clone(), logger.clone()).await;
                    }
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        );
    }

}

impl Actor for YpfRuta {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.intentar_conectar_peers(ctx);
        self.escuchar_peers(ctx);

        self.escuchar_estaciones(ctx);
        self.procesar_ventas(ctx);
        self.escuchar_empresas(ctx);
    }
}
