use actix::{Actor, Context, Handler, Message};
use actix::prelude::*;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_util::codec::{FramedRead, LinesCodec};
use std::net::SocketAddr;
use tokio::net::TcpStream;
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

// Mensaje interno para reenviar una línea cuando la conexión ya esté registrada
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Reenviar(pub String);

// Mensaje para pasar por el anillo
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Eleccion {
    pub aspirantes_ids: Vec<usize>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct NotificarLider {
    pub id_lider: usize,
    pub id_iniciador: usize,
}

pub struct Estacion {
    id: usize,
    port: u16,
    lider_actual: Option<usize>,
    estaciones_cercanas: Vec<ConexionEstacion>,
    siguiente_estacion: Option<SocketAddr>,
    total_estaciones: usize,
    todas_las_estaciones: Vec<SocketAddr>,
    primer_anillo_realizado: bool
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
            primer_anillo_realizado : false
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
    async fn intentar_conectar(
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

    fn enviar_a_siguiente(&self, ctx: &mut Context<Self>, mensaje: String) {
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
        
        if es_ultima && !self.primer_anillo_realizado{
            self.primer_anillo_realizado = true;
            println!("[{}] Soy la última estación, iniciando ronda de mensajes", self.id);
            let siguiente_estacion = self.siguiente_estacion;
            let id_estacion = self.id;
            let addr_self = ctx.address();
            actix_rt::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                let mensaje = Eleccion {
                    aspirantes_ids: vec![id_estacion],
                };        
                if let Some(siguiente) = siguiente_estacion {
                    let ids_str: Vec<String> = mensaje.aspirantes_ids.iter().map(|id| id.to_string()).collect();
                    let mensaje_serializado = format!("ANILLO:{}", ids_str.join(","));
                    addr_self.do_send(Reenviar(mensaje_serializado));
                    println!("[{}] enviando mensaje inicial (via Reenviar) a siguiente estación en {}", id_estacion, siguiente);
                }
            });
        }
    }
}

impl Handler<Reenviar> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Reenviar, ctx: &mut Context<Self>) {
        self.enviar_a_siguiente(ctx, msg.0);
    }
}

impl Handler<Eleccion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, ctx: &mut Context<Self>) {
        println!("[{}] recibió mensaje del anillo: {:?}", self.id, msg.aspirantes_ids);
        self.primer_anillo_realizado = true;

        if msg.aspirantes_ids.contains(&self.id) {
            if let Some(nuevo_lider) = msg.aspirantes_ids.iter().max().copied() {
                println!(
                    "[{}] Detecté que mi id está en la lista, el nuevo líder es {}.",
                    self.id, nuevo_lider
                );

                // Enviamos el mensaje solo si encontramos líder
                ctx.address().do_send(NotificarLider { id_lider: nuevo_lider, id_iniciador: self.id });
            }
            return;
        }

        let mut nuevos_aspirantes = msg.aspirantes_ids.clone();
        nuevos_aspirantes.push(self.id);
        println!("[{}] agregue mi id. Nuevos aspirantes: {:?}", self.id, nuevos_aspirantes);

        let ids_str: Vec<String> = nuevos_aspirantes.iter().map(|id| id.to_string()).collect();
        let mensaje_serializado = format!("ANILLO:{}", ids_str.join(","));
        self.enviar_a_siguiente(ctx, mensaje_serializado);
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
                } else if line.starts_with("LIDER:") {
                    let contenido = line.strip_prefix("LIDER:").unwrap_or(&line).trim();
                    let partes: Vec<&str> = contenido.split(',').collect();
                    if partes.len() == 2 {
                        if let (Ok(id_lider), Ok(id_iniciador)) = (partes[0].parse::<usize>(), partes[1].parse::<usize>()) {
                            server_addr.do_send(NotificarLider { id_lider, id_iniciador });
                        }
                    }
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
                } else if line.starts_with("LIDER:") {
                    let contenido = line.strip_prefix("LIDER:").unwrap_or(&line).trim();
                    let partes: Vec<&str> = contenido.split(',').collect();
                    if partes.len() == 2 {
                        if let (Ok(id_lider), Ok(id_iniciador)) = (partes[0].parse::<usize>(), partes[1].parse::<usize>()) {
                            server_addr.do_send(NotificarLider { id_lider, id_iniciador });
                        }
                    }
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

impl Handler<NotificarLider> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: NotificarLider, ctx: &mut Context<Self>) {
        if self.id == msg.id_iniciador && self.lider_actual == Some(msg.id_lider){
            println!(
                "[{}] 🔁 Mensaje de líder {} completó el ciclo, fin de propagación.",
                self.id, msg.id_lider
            );
            return;
        }

        self.lider_actual = Some(msg.id_lider);

        println!(
            "[{}] Mi nuevo lider es : {} ",
            self.id, msg.id_lider
        );

        if self.id != msg.id_lider {
            if let Some(lider_addr) = self.todas_las_estaciones.get(msg.id_lider).copied()
            {
                if !self.estaciones_cercanas.iter().any(|c| c.peer_addr == lider_addr)
                {
                    println!(
                        "[{}] intentando conectarme al nuevo líder en {}...",
                        self.id, lider_addr
                    );

                    let addr_self = ctx.address();
                    let self_id = self.id;

                    ctx.spawn(
                        actix::fut::wrap_future(async move {
                            match Estacion::intentar_conectar(lider_addr, addr_self.clone(), self_id).await {
                                Ok(_) => println!("[{}] ✅ conexión establecida con el líder {}", self_id, msg.id_lider),
                                Err(_) => println!("[{}] ❌ no se pudo conectar con el líder {}", self_id, msg.id_lider),
                            }
                        })
                            .map(|_, _, _| ()),
                    );
                }
            }
        } else {
            println!(
                "[{}] Soy el nuevo líder, no necesito conectarme a mí mismo.",
                self.id
            );
        }

        // Reenviamos el mensaje al siguiente en el anillo
        let mensaje_serializado = format!("LIDER:{},{}", msg.id_lider, msg.id_iniciador);
        self.enviar_a_siguiente(ctx, mensaje_serializado);
    }
}