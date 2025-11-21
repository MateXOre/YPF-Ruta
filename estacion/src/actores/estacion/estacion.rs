use actix::{Actor, Context, Addr};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpStream, TcpListener};
use actix::prelude::*;
use crate::actores::estacion::messages::*;
use crate::actores::estacion_cercana::EstacionCercana;
use crate::actores::estacion_cercana::Enviar;
use crate::actores::estacion::io::{handle_stream_incoming, handle_stream_outgoing};
use crate::actores::surtidor::surtidor::Surtidor;

use std::collections::VecDeque;


// Estructura para guardar información de una conexión
// #[derive(Clone)]
// pub struct ConexionEstacion {
//     pub peer_addr: SocketAddr,
//     pub actor: Addr<EstacionCercana>,
//     pub estacion_id: usize
// }

pub struct Estacion {
    pub(crate) desconectada: bool,
    pub(crate) id: usize,
    pub(crate) port: u16,
    pub(crate) lider_actual: Option<usize>,                     // id de la estación líder actual
    pub(crate) siguiente_estacion: usize,               // id de la siguiente estación
    pub(crate) estaciones_cercanas: HashMap<usize, Addr<EstacionCercana>>, // id_estacion, addr actor estacion cercana
    pub(crate) total_estaciones: usize,
    pub(crate) todas_las_estaciones: HashMap<usize, SocketAddr>,   // id_estacion, socketaddr
    pub(crate) primer_anillo_realizado: bool,
    pub(crate) ventas_a_confirmar: HashMap<usize, usize>, // id_venta, id_surtidor
    pub(crate) surtidores: HashMap<usize,Addr<Surtidor>>,
    pub(crate) max_surtidores: usize,
    pub cola_espera: VecDeque<AceptarCliente>,
}

impl Estacion {
    pub fn new(index_estacion: usize, estaciones: Vec<SocketAddr>) -> Self {
        let siguiente = if index_estacion + 1 < estaciones.len() {
            index_estacion + 1
        } else {
            0
        };

        Self {
            desconectada: false,
            id: index_estacion,
            port: estaciones[index_estacion].port(),
            lider_actual: None,
            estaciones_cercanas: HashMap::new(),
            siguiente_estacion: siguiente,
            total_estaciones: estaciones.len(),
            todas_las_estaciones: estaciones.into_iter().enumerate().collect(),
            primer_anillo_realizado : false,
            ventas_a_confirmar: HashMap::new(),
            surtidores: HashMap::new(),
            max_surtidores: 4,
            cola_espera: VecDeque::new()
        }
    }

    async fn connect_and_register(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
        id_destino: usize,
    ) {
        let _ = Estacion::intentar_conectar(target, actor_addr, id, id_destino).await;
    }

    /// Intenta conectar a una estación (no bloquea, ejecuta en background)
    pub(crate) async fn intentar_conectar(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
        id_destino: usize,
    ) -> Result<(), std::io::Error> {
        match TcpStream::connect(target).await {
            Ok(stream) => {
                println!("[{}] ✅ conectado a {} con dirección:{}", id, id_destino, target);
                println!(
                    "[{}] ✅ conectado → local={} → remoto={}",
                    id,
                    stream.local_addr()?,
                    stream.peer_addr()?
                );

                let addr_clone = actor_addr.clone();
                if let Err(e) = handle_stream_outgoing(stream, id, addr_clone, id_destino).await {
                    eprintln!("[{}] error al manejar conexión con {}: {:?}", id, target, e);
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("[{}] ❌ no pudo conectar a {}: {}", id, target, e);
                Err(e)
            }
        }
    }

    /// 
    pub(crate) fn enviar_a_siguiente(&self, ctx: &mut Context<Self>, mensaje: Vec<u8>) {
        println!("[{}] 🔁 reenviando mensaje", self.id);

        let siguiente = if self.estaciones_cercanas.get(&self.siguiente_estacion).is_some() {
            self.estaciones_cercanas.get(&self.siguiente_estacion).unwrap().clone()
        } else {
            println!("[{}] La siguiente estación {} no está conectada, no se puede reenviar el mensaje", self.id, self.siguiente_estacion);
            ctx.address().do_send(EstacionDesconectada{estacion_id: self.siguiente_estacion.clone(), mensaje: mensaje.clone()});
            return;
        };
        println!("[{}] 🔁 Mensaje reenviado exitosamente", self.id);
        siguiente.do_send(Enviar { bytes: mensaje.clone() });

    }

    pub(crate) fn id_siguiente_estacion(&self) -> usize {
        (self.id + 1) % (self.todas_las_estaciones.len())
    }

    pub(crate) fn buscar_estacion_lider(&self) -> Option<Addr<EstacionCercana>> {
        if let Some(lider) = self.lider_actual {
            if let Some(conexion) = self.estaciones_cercanas.get(&lider) {
                return Some(conexion.clone());
            }
        }
        None
    }

    pub(crate) fn buscar_estacion_por_id(&self, id: usize) -> Option<Addr<EstacionCercana>> {
        if let Some(conexion) = self.estaciones_cercanas.get(&id) {
            return Some(conexion.clone());
        }
        None
    }
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let port = self.port;
        let id = self.id;
        let addr_self = ctx.address();
        let addr_self_clone = addr_self.clone();
        println!("[{}] Soy la estacion {} la siguiente es la estacion {}", id, id, self.siguiente_estacion);
        println!("[{}] escuchando conexiones de clientes en 127.0.0.1:{}", id, port + 1000);
        // correr listener en background
        actix_rt::spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port + 1000)).await.unwrap();
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        println!("[{}] conexión de cliente entrante desde {:?}", id, peer_addr);
                        // se inicia un actor de surtidor por cada conexión entrante
                        addr_self_clone.do_send(AceptarCliente { stream, peer_addr});
                        /*let estacion_addr = addr_self_clone.clone();
                        let id_surtidor = rand::random::<u64>() as usize;
                        actix_rt::spawn(async move {
                            let estacion: Addr<Estacion> = estacion_addr.clone();
                            let surtidor = Surtidor::new(id_surtidor, estacion_addr, stream, id);
                            let surtidor_addr = surtidor.start();
                            estacion.do_send(HabilitarSurtidor {
                                surtidor_id: id_surtidor,
                                surtidor_addr,
                            });
                        });*/
                    }
                    Err(e) => {
                        eprintln!("Error al aceptar conexión de cliente: {:?}", e);
                    }
                }
            }
        });


        println!("[{}] escuchando conexiones de estaciones en 127.0.0.1:{}", id, port);

        // correr listener en background
        actix_rt::spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        println!("[{}] conexión de estación entrante desde {:?}", id, peer_addr);
                        // Spawn en un task separado para no bloquear el accept de nuevas conexiones
                        let addr_clone = addr_self.clone();
                        actix_rt::spawn(async move {
                            if let Err(e) = handle_stream_incoming(stream, id, addr_clone).await {
                                eprintln!("Error manejando conexión entrante: {:?}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error al aceptar conexión: {:?}", e);
                    }
                }
            }
        });


        // Conectar con la siguiente estación después de crear el hilo

        let addr_self_clone = ctx.address();
        let sig_addr = self.todas_las_estaciones.get(&self.siguiente_estacion).unwrap().clone();
        let sig_id = self.siguiente_estacion;
        println!("Conectando con siguiente estación {}", sig_id);
        actix_rt::spawn(async move {
            Estacion::connect_and_register(sig_addr, addr_self_clone, id, sig_id).await;
        });
        
    }
}

