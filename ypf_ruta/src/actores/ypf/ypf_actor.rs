use crate::actores::estacion::estacion_actor::Estacion;
use crate::actores::estacion::messages::{ResultadoVentas, Solicitud};
use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::gestor::messages::ValidarVenta;
use crate::actores::peer::messages::VentaRegistrada;
use crate::actores::peer::ypf_peer::YpfPeer;
use crate::actores::ypf::messages::ConexionEntrante;
use actix::ActorFutureExt;
use actix::{Actor, Addr, AsyncContext, WrapFuture};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;

const DIRECCION_IP: &str = "127.0.0.1";
const OFFSET_ESTACIONES: usize = 10000;

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
}

impl YpfRuta {
    pub fn new(
        id: usize,
        puerto: usize,
        lider: Option<usize>,
        peers: HashMap<usize, SocketAddr>,
        gestor_addr: Addr<Gestor>,
    ) -> Self {
        let ypf_peers = HashMap::new();
        println!("YpfRuta {}: Creando instancia.", id);

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

        let fut = async move { YpfPeer::new(peer_id, self_id, socket, addr_opt, self_addr).await };

        let fut = fut.into_actor(self).map(move |peer, act, _ctx| {
            let addr = peer.start();
            act.ypf_peers.insert(peer_id, addr);
            println!(
                "YpfRuta {}: Peer {} registrado desde conexión entrante",
                act.id, peer_id
            );
        });

        ctx.spawn(fut);
    }

    fn intentar_conectar_peers(&mut self, ctx: &mut actix::Context<Self>) {
        println!("YpfRuta {}: Intentando conectar a peers...", self.id);
        let peers = self.peer_addrs.clone();
        let self_id = self.id.clone();

        for (peer_id, addr) in peers {
            let self_addr = ctx.address().clone();
            let fut = async move {
                YpfPeer::new(peer_id.clone(), self_id, None, Some(addr), self_addr).await
            };
            let id = peer_id.clone();

            let fut = fut.into_actor(self).map(move |peer, act, _ctx| {
                let addr = peer.start();
                act.ypf_peers.insert(id.clone(), addr);
                println!("YpfRuta {}: Peer {} iniciado.", act.id, id);
            });

            ctx.spawn(fut);
        }
    }

    fn escuchar_peers(&mut self, ctx: &mut actix::Context<Self>) {
        println!(
            "YpfRuta {}: Escuchando conexiones entrantes de peers...",
            self.id
        );
        let puerto = self.puerto as u16;
        let self_id = self.id;
        let self_addr = ctx.address();

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
                                println!(
                                    "YpfRuta {}: Línea recibida del socket entrante: {}",
                                    self_id,
                                    line.trim_end()
                                );
                                // parsear "ID_LOCAL:123\n"
                                if let Some(id_str) = line.strip_prefix("ID_LOCAL:") {
                                    if let Ok(peer_id) = id_str.trim().parse::<usize>() {
                                        println!(
                                            "YpfRuta {}: Conexión entrante del peer {}",
                                            self_id, peer_id
                                        );

                                        // obtener el socket interno del reader
                                        let socket = reader.into_inner();

                                        // enviar el socket al actor YpfRuta para que lo maneje
                                        self_addr.do_send(ConexionEntrante { peer_id, socket });
                                    } else {
                                        println!(
                                            "YpfRuta {}: ID inválido recibido: {}",
                                            self_id, line
                                        );
                                    }
                                } else {
                                    println!(
                                        "YpfRuta {}: Formato de mensaje inválido: {}",
                                        self_id, line
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!("YpfRuta {}: Error leyendo socket: {}", self_id, e);
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
            println!(
                "YpfRuta {}: SOY LIDER, Escuchando conexiones entrantes de estaciones líderes...",
                self.id
            );
        } else {
            return;
        }

        let puerto = (self.puerto + OFFSET_ESTACIONES) as u16;
        let self_id = self.id;
        let self_addr = ctx.address();

        ctx.spawn(
            async move {
                let listener =
                    if let Ok(l) = tokio::net::TcpListener::bind((DIRECCION_IP, puerto)).await {
                        l
                    } else {
                        println!("No se pudo crear el listener para conexión de estaciones");
                        return;
                    };

                println!(
                    "YpfRuta {}: Listener de estaciones activo en puerto {}",
                    self_id, puerto
                );

                loop {
                    if let Ok((socket, peer_addr)) = listener.accept().await {
                        println!(
                            "YpfRuta {}: Nueva conexión de estación desde {:?}",
                            self_id, peer_addr
                        );

                        match Estacion::new(socket, self_addr.clone()).await {
                            Ok(estacion_addr) => estacion_addr.start(),
                            Err(_) => continue,
                        };

                        println!(
                            "YpfRuta {}: Actor de Estacion creado para {:?}",
                            self_id, peer_addr
                        );
                    }
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        );
    }

    fn procesar_ventas(&mut self, ctx: &mut actix::Context<Self>) {
        // Tarea periódica que procesa la cola de ventas
        ctx.run_interval(std::time::Duration::from_millis(100), |act, _ctx| {
            if let Some((estacion_addr, ventas)) = act.ventas_por_confirmar.pop_front() {
                println!(
                    "YpfRuta {}: Procesando {} ventas de la cola",
                    act.id,
                    ventas.len()
                );

                // Procesar ventas y guardar resultados
                let gestor = act.gestor_addr.clone();
                let ypf_id = act.id;
                let ypfs_addr: Vec<Addr<YpfPeer>> = act.ypf_peers.values().cloned().collect();

                // procesar todas las ventas
                actix::spawn(async move {
                    let mut ventas_aprobadas = Vec::new();
                    let mut resultados_por_estacion = HashMap::new();
                    for (estacion, ventas_por_surtidor) in ventas {
                        let mut resultado_por_surtidor = HashMap::new();
                        for (surtidor, vec_ventas) in ventas_por_surtidor {
                            let mut resultado_ventas = Vec::new();
                            for venta in vec_ventas {
                                println!(
                                    "YpfRuta {}: Validando venta {} en el gestor",
                                    ypf_id, venta.id_venta
                                );

                                // Enviar al gestor y esperar respuesta
                                match gestor.send(ValidarVenta(venta.clone())).await {
                                    Ok(aprobada) => {
                                        println!(
                                            "YpfRuta {}: Venta {} - Resultado: {}",
                                            ypf_id, venta.id_venta, aprobada
                                        );

                                        if aprobada {
                                            ventas_aprobadas.push(venta.clone());
                                        }

                                        resultado_ventas.push((venta.id_venta.clone(), aprobada));
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "YpfRuta {}: Error validando venta {}: {:?}",
                                            ypf_id, venta.id_venta, e
                                        );
                                    }
                                }
                            }
                            resultado_por_surtidor.insert(surtidor, resultado_ventas);
                        }

                        resultados_por_estacion.insert(estacion, resultado_por_surtidor);
                    }

                    estacion_addr.do_send(ResultadoVentas {
                        ventas: resultados_por_estacion,
                    });

                    // Enviar cada venta aprobada individualmente a todos los peers
                    for venta in ventas_aprobadas {
                        println!(
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
}

impl Actor for YpfRuta {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("YpfRuta {} iniciado en el puerto {}.", self.id, self.puerto);

        // Conexion a peers
        self.intentar_conectar_peers(ctx);
        self.escuchar_peers(ctx);

        // Abrimos un listener para empresas

        // Abrimos un listener para estaciones lideres
        self.escuchar_estaciones(ctx);
        // Iniciar procesador de cola de ventas
        self.procesar_ventas(ctx);
    }
}
