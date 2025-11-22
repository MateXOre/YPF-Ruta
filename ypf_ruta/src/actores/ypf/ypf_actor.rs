use crate::actores::gestor::gestor_actor::Gestor;
use crate::actores::peer::ypf_peer::YpfPeer;
use actix::ActorFutureExt;
use actix::{Actor, Addr, AsyncContext, WrapFuture};
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::actores::ypf::messages::{ConexionEntrante, NuevoLider};

pub struct YpfRuta {
    pub(crate) id: usize,
    puerto: usize,
    pub(crate) lider: Option<usize>,
    pub(crate) ypf_peers: HashMap<usize, Addr<YpfPeer>>,
    peer_addrs: HashMap<usize, SocketAddr>,
    gestor_addr: Addr<Gestor>,
    // Estado para algoritmo Bully
    pub(crate) en_eleccion: bool,
    pub(crate) respuestas_recibidas: usize,
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
        YpfRuta {
            id,
            puerto,
            lider,
            ypf_peers,
            peer_addrs: peers,
            gestor_addr,
            en_eleccion: false,
            respuestas_recibidas: 0,
        }
    }

    fn intentar_conectar_peers(&mut self, ctx: &mut actix::Context<Self>) {
        println!("YpfRuta {}: Intentando conectar a peers...", self.id);
        let peers = self.peer_addrs.clone();
        let self_id = self.id.clone();
        let lider = self.lider.clone();

        for (peer_id, addr) in peers {
            let self_addr = ctx.address().clone();
            let fut =
                async move { YpfPeer::new(peer_id.clone(), self_id, None, Some(addr), self_addr).await };
            let id = peer_id.clone();

            let fut = fut.into_actor(self).map(move |peer, act, _ctx| {
                let addr = peer.start();

                if let Some(lider_id) = lider {
                    println!("YpfRuta {}: Enviando información de líder al peer {}", self_id.clone(), peer_id.clone());
                    
                    addr.do_send(NuevoLider { id: lider_id });
                }

                act.ypf_peers.insert(id.clone(), addr);
                println!("YpfRuta {}: Peer {} iniciado.", act.id, id);

                
            });

            ctx.spawn(fut);
        }
    }

    fn escuchar_peers(&mut self, ctx: &mut actix::Context<Self>) {
        println!("YpfRuta {}: Escuchando conexiones entrantes de peers...", self.id);
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
                                // parsear "ID_LOCAL:123\n"
                                if let Some(id_str) = line.strip_prefix("ID_LOCAL:") {
                                    if let Ok(peer_id) = id_str.trim().parse::<usize>() {
                                        println!("YpfRuta {}: Conexión entrante del peer {}", self_id, peer_id);
                                        
                                        // obtener el socket interno del reader
                                        let socket = reader.into_inner();
                                        
                                        // enviar el socket al actor YpfRuta para que lo maneje
                                        self_addr.do_send(ConexionEntrante {
                                            peer_id,
                                            socket,
                                        });
                                    } else {
                                        println!("YpfRuta {}: ID inválido recibido: {}", self_id, line);
                                    }
                                } else {
                                    println!("YpfRuta {}: Formato de mensaje inválido: {}", self_id, line);
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
    }
}

