use actix::{Actor, Context, Addr};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Add;
use tokio::net::{TcpStream, TcpListener};
use actix::prelude::*;
use crate::actores::estacion::{self, messages::*};
use crate::actores::estacion_cercana::EstacionCercana;
use crate::actores::estacion::io::{handle_stream_incoming, handle_stream_outgoing};
use crate::actores::surtidor::actor::Surtidor;

// Estructura para guardar información de una conexión
#[derive(Clone)]
pub struct ConexionEstacion {
    pub peer_addr: SocketAddr,
    pub actor: Addr<EstacionCercana>,
}

pub struct Estacion {
    pub(crate) id: usize,
    pub(crate) port: u16,
    pub(crate) lider_actual: Option<usize>,
    pub(crate) estaciones_cercanas: Vec<ConexionEstacion>,
    pub(crate) siguiente_estacion: Option<SocketAddr>,
    pub(crate) total_estaciones: usize,
    pub(crate) todas_las_estaciones: Vec<SocketAddr>,
    pub(crate) primer_anillo_realizado: bool,

    pub(crate) surtidores: HashMap<usize,Addr<Surtidor>>,
}

impl Estacion {
    pub fn new(index_estacion: usize, estaciones: Vec<SocketAddr>) -> Self {
        let siguiente = if index_estacion + 1 < estaciones.len() {
            Some(estaciones[index_estacion + 1])
        } else {
            Some(estaciones[0])
        };

        Self {
            id: index_estacion,
            port: estaciones[index_estacion].port(),
            lider_actual: None,
            estaciones_cercanas: Vec::new(),
            siguiente_estacion: siguiente,
            total_estaciones: estaciones.len(),
            todas_las_estaciones: estaciones,
            primer_anillo_realizado : false,
            surtidores: HashMap::new(),
        }
    }

    async fn connect_and_register(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
    ) {
        let _ = Estacion::intentar_conectar(target, actor_addr, id).await;
    }

    /// Intenta conectar a una estación (no bloquea, ejecuta en background)
    pub(crate) async fn intentar_conectar(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
    ) -> Result<(), std::io::Error> {
        match TcpStream::connect(target).await {
            Ok(stream) => {
                println!("[{}] ✅ conectado a {}", id, target);

                let addr_clone = actor_addr.clone();
                actix_rt::spawn(async move {
                    if let Err(e) = handle_stream_outgoing(stream, id, addr_clone).await {
                        eprintln!("[{}] error al manejar conexión con {}: {:?}", id, target, e);
                    }
                });

                Ok(())
            }
            Err(e) => {
                eprintln!("[{}] ❌ no pudo conectar a {}: {}", id, target, e);
                Err(e)
            }
        }
    }

    pub(crate) fn enviar_a_siguiente(&self, ctx: &mut Context<Self>, mensaje: String) {
        if let Some(siguiente) = &self.siguiente_estacion {
            if let Some(conexion) = self.estaciones_cercanas.iter().find(|c| c.peer_addr == *siguiente) {
                conexion.actor.do_send(crate::actores::estacion_cercana::ConectarEstacion(mensaje.clone()));
                println!("[{}] 🔁 reenviando mensaje a {}", self.id, siguiente);
            } else {
                println!("[{}] ❌ sin conexión a {}, intentando reconectar...", self.id, siguiente);

                let siguiente_clone = *siguiente;
                let addr_self = ctx.address();
                let self_id = self.id;
                let mensaje_clone = mensaje.clone();

                // Intentar reconectar en background; si tiene éxito, pedir reenvío (Reenviar)
                ctx.spawn(
                    actix::fut::wrap_future(async move {
                        if Estacion::intentar_conectar(siguiente_clone, addr_self.clone(), self_id).await.is_ok() {
                            Some(mensaje_clone)
                        } else {
                            None
                        }
                    })
                        .map(|maybe_msg, _act: &mut Estacion, ctx: &mut Context<Estacion>| {
                            if let Some(mensaje) = maybe_msg {
                                // cuando la reconexión haya registrado la conexión (AgregarEstacion),
                                // recibiremos Reenviar y volveremos a intentar enviar.
                                ctx.address().do_send(Reenviar(mensaje));
                            }
                        }),
                );
            }
        }
    }
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let port = self.port;
        let id = self.id;
        let addr_self = ctx.address();
        let addr_self_clone = addr_self.clone();

        println!("[{}] escuchando conexiones de clientes en 127.0.0.1:{}", id, port + 1000);
        // correr listener en background
        actix_rt::spawn(async move {
            let listener = TcpListener::bind(("127.0.0.1", port + 1000)).await.unwrap();
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        println!("[{}] conexión entrante desde {:?}", id, peer_addr);
                        // se inicia un actor de surtidor por cada conexión entrante
                        let estacion_addr = addr_self_clone.clone();
                        let id_surtidor = rand::random::<u64>() as usize;
                        actix_rt::spawn(async move {
                            let estacion: Addr<Estacion> = estacion_addr.clone();
                            let surtidor = Surtidor::new(id_surtidor, estacion_addr, stream, id as i32);
                            let surtidor_addr = surtidor.start();
                            estacion.do_send(HabilitarSurtidor {
                                surtidor_id: id_surtidor,
                                surtidor_addr,
                            });
                        });
                    }
                    Err(e) => {
                        eprintln!("Error al aceptar conexión: {:?}", e);
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
                        println!("[{}] conexión entrante desde {:?}", id, peer_addr);
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
        if let Some(siguiente) = self.siguiente_estacion {
            let addr_self_clone = ctx.address();
            actix_rt::spawn(async move {
                Estacion::connect_and_register(siguiente, addr_self_clone, id).await;
            });
        }
    }
}