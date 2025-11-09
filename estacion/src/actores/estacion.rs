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

// Estructura para guardar información de una conexión
#[derive(Clone)]
pub struct ConexionEstacion {
    pub peer_addr: SocketAddr,
    pub actor: Addr<EstacionCercana>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AgregarEstacion {
    pub peer: Addr<EstacionCercana>,
    pub peer_addr: SocketAddr,
}



// Mensaje para pasar por el anillo
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Eleccion {
    pub aspirantes_ids: Vec<usize>,
}

pub struct Estacion {
    id: usize,
    port: u16,
    estaciones_cercanas: Vec<ConexionEstacion>,
    siguiente_estacion: Option<SocketAddr>,
    total_estaciones: usize,
    todas_las_estaciones: Vec<SocketAddr>
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
            estaciones_cercanas: Vec::new(),
            siguiente_estacion: siguiente,
            total_estaciones: estaciones.len(),
            todas_las_estaciones: estaciones,
        }
    }

    async fn connect_and_register(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
    ) {
        Estacion::intentar_conectar(target, actor_addr, id);
    }

    /// Intenta conectar a una estación (no bloquea, ejecuta en background)
    fn intentar_conectar(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
    ) {
        actix_rt::spawn(async move {
            match TcpStream::connect(target).await {
                Ok(stream) => {
                    println!("[{}] conectado a {}", id, target);
                    // Manejar la conexión en un task separado (no bloquea)
                    let addr_clone = actor_addr.clone();
                    actix_rt::spawn(async move {
                        if let Err(e) = handle_stream_outgoing(stream, id, addr_clone).await {
                            eprintln!("[{}] error al manejar conexión con {}: {:?}", id, target, e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("[{}] no pudo conectar a {}: {}", id, target, e);
                }
            }
        });
    }
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let port = self.port;
        let id = self.id;
        println!("[{}] escuchando en 127.0.0.1:{}", id, port);

        let addr_self = ctx.address();

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

impl Handler<AgregarEstacion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: AgregarEstacion, ctx: &mut Context<Self>) {
        println!("[{}] agregando estación conectada desde {}", self.id, msg.peer_addr);
        self.estaciones_cercanas.push(ConexionEstacion {
            peer_addr: msg.peer_addr,
            actor: msg.peer,
        });
        
        // Si es la última estación (id = total_estaciones - 1) y tiene su conexión lista, iniciar la ronda
        let es_ultima = self.id == self.total_estaciones - 1;
        
        if es_ultima {
            println!("[{}] Soy la última estación, iniciando ronda de mensajes", self.id);
            let siguiente_estacion = self.siguiente_estacion;
            let estaciones_cercanas_clone = self.estaciones_cercanas.clone();
            let id_estacion = self.id;
            actix_rt::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                let mensaje = Eleccion {
                    aspirantes_ids: vec![id_estacion],
                };        
                if let Some(siguiente) = siguiente_estacion {
                    if let Some(conexion) = estaciones_cercanas_clone.iter().find(|c| {
                        c.peer_addr == siguiente
                    }) {
                        let ids_str: Vec<String> = mensaje.aspirantes_ids.iter().map(|id| id.to_string()).collect();
                        let mensaje_serializado = format!("ANILLO:{}", ids_str.join(","));
                        conexion.actor.do_send(crate::actores::estacion_cercana::ConectarEstacion(mensaje_serializado));
                        println!("[{}] enviando mensaje inicial a siguiente estación en {}", id_estacion, siguiente);
                    } else {
                        println!("[{}] no encontró conexión a la siguiente estación {}", id_estacion, siguiente);
                    }
                }
            });
        }
    }
}

impl Handler<Eleccion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, ctx: &mut Context<Self>) {
        println!("[{}] recibió mensaje del anillo: {:?}", self.id, msg.aspirantes_ids);
        

        if msg.aspirantes_ids.contains(&self.id) {
            println!("[{}] Detecte que mi id esta en la lista, entonces les aviso a todos quien es el nuevo lider.", self.id);
            return;
        }
        
        let mut nuevos_aspirantes = msg.aspirantes_ids.clone();
        nuevos_aspirantes.push(self.id);
        println!("[{}] agregue mi id. Nuevos aspirantes: {:?}", self.id, nuevos_aspirantes);
        
        if let Some(siguiente) = &self.siguiente_estacion {
            if let Some(conexion) = self.estaciones_cercanas.iter().find(|c| {
                c.peer_addr == *siguiente
            }) {
                let ids_str: Vec<String> = nuevos_aspirantes.iter().map(|id| id.to_string()).collect();
                let mensaje_serializado = format!("ANILLO:{}", ids_str.join(","));
                conexion.actor.do_send(crate::actores::estacion_cercana::ConectarEstacion(mensaje_serializado));
                println!("[{}] reenviando mensaje a siguiente estación en {}", self.id, siguiente);
            } else {
                println!("[{}] intentando reconectar con estación {}", self.id, siguiente);
                let siguiente_clone = *siguiente;
                let addr_self = ctx.address();
                let mensaje_para_reenviar = Eleccion {
                    aspirantes_ids: nuevos_aspirantes,
                };
                Estacion::intentar_conectar(
                    siguiente_clone,
                    addr_self.clone(),
                    self.id,
                );
                addr_self.do_send(mensaje_para_reenviar);
            }
        }
    }
}

async fn handle_stream_incoming(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Obtener la dirección del peer antes de dividir el stream
    let peer_addr = stream.peer_addr()?;
    
    let (r, mut w) = stream.into_split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // crear actor
    let conn = EstacionCercana {
        estacion_id: format!("{}(incoming)", id),
        tx: tx.clone(),
    }
    .start();

    // registrar en el servidor con la información del socket
    server_addr.do_send(AgregarEstacion { 
        peer: conn.clone(),
        peer_addr,
    });

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
            Ok(line) => {
                // Si el mensaje es del anillo, convertirlo en Eleccion y enviarlo a la Estación
                if line.starts_with("ANILLO:") {
                    let contenido = line.strip_prefix("ANILLO:").unwrap_or(&line).trim();
                    // Parsear los IDs separados por comas
                    let aspirantes_ids: Vec<usize> = contenido
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    server_addr.do_send(Eleccion { aspirantes_ids });
                } else {
                    // Mensaje normal, enviarlo al actor EstacionCercana
                    conn.do_send(crate::actores::estacion_cercana::RespuestaConexion(line));
                }
            }
            Err(e) => {
                eprintln!("Error lectura entrante en {}: {:?}", id, e);
                break;
            }
        }
    }

    Ok(())
}

async fn handle_stream_outgoing(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Obtener la dirección del peer antes de dividir el stream
    let peer_addr = stream.peer_addr()?;
    
    let (r, mut w) = stream.into_split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    let conn = EstacionCercana {
        estacion_id: format!("{}(outgoing)", id),
        tx: tx.clone(),
    }
    .start();

    // registrar en el servidor con la información del socket
    server_addr.do_send(AgregarEstacion { 
        peer: conn.clone(),
        peer_addr,
    });

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
            Ok(line) => {
                // Si el mensaje es del anillo, convertirlo en Eleccion y enviarlo a la Estación
                if line.starts_with("ANILLO:") {
                    let contenido = line.strip_prefix("ANILLO:").unwrap_or(&line).trim();
                    // Parsear los IDs separados por comas
                    let aspirantes_ids: Vec<usize> = contenido
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                    server_addr.do_send(Eleccion { aspirantes_ids });
                } else {
                    // Mensaje normal, enviarlo al actor EstacionCercana
                    conn.do_send(crate::actores::estacion_cercana::RespuestaConexion(line));
                }
            }
            Err(e) => {
                eprintln!("Error lectura saliente en {}: {:?}", id, e);
                break;
            }
        }
    }

    Ok(())
}