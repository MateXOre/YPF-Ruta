use crate::actores::estacion::messages::InformarVenta;

use crate::actores::estacion::Eleccion;
use crate::actores::estacion::Estacion;
use crate::actores::estacion::NotificarLider;
use crate::actores::estacion::{
    deserialize_message, ConfirmarTransacciones, EnviarASiguiente, LiderCaido, MessageType,
    ProcesarMensaje, DesconexionDetectada
};
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler, Message};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

// ===== Mensajes =====

#[derive(Message)]
#[rtype(result = "()")]
pub struct Enviar {
    pub bytes: Vec<u8>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct EstacionCercanaCerroConexion;

#[derive(Message)]
#[rtype(result = "()")]
pub struct CerrarConexion;

pub struct EstacionCercana {
    pub estacion_id: usize,
    pub estacion_local: Addr<Estacion>,
    pub socket_estacion_cercana: UnboundedSender<Vec<u8>>,
    pub reader: Option<OwnedReadHalf>,
    pub estacion_local_id: usize,
    pub desconectado: bool,
    reader_task: Option<tokio::task::JoinHandle<()>>, // Guardamos el handle del task de lectura para poder abortarlo
}

impl Actor for EstacionCercana {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let estacion_addr = ctx.address();

        let reader = self.reader.take().expect("reader debería estar");

        let reader_task = EstacionCercana::read_from_socket(
            reader,
            self.estacion_local.clone(),
            self.estacion_id,
            estacion_addr,
        );
        self.reader_task = Some(reader_task);
    }
}


impl Handler<Enviar> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: Enviar, _ctx: &mut Context<Self>) {
        println!(
            "Enviando mensaje a la estación cercana {}",
            self.estacion_id
        );
        let buf = msg.bytes.clone();
        self.enviar_por_socket(buf);
    }
}

impl Handler<EstacionCercanaCerroConexion> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, _msg: EstacionCercanaCerroConexion, _ctx: &mut Context<Self>) {
        println!(
            "[{}] Marcando estación {} como desconectada",
            self.estacion_local_id, self.estacion_id
        );

        self.desconectado = true;

        self.estacion_local.do_send(DesconexionDetectada{estacion_id: self.estacion_id});
    }
}

impl Handler<Eleccion> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, _ctx: &mut Context<Self>) {
        let buf = msg.to_bytes();
        self.enviar_por_socket(buf);
    }
}

impl Handler<NotificarLider> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: NotificarLider, _ctx: &mut Context<Self>) {
        let buf = msg.to_bytes();
        self.enviar_por_socket(buf);
    }
}

impl Handler<InformarVenta> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: InformarVenta, _ctx: &mut Context<Self>) {
        let buf = msg.to_bytes();
        self.enviar_por_socket(buf);
    }
}

impl Handler<ConfirmarTransacciones> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: ConfirmarTransacciones, _ctx: &mut Context<Self>) {
        let buf = msg.to_bytes();
        self.enviar_por_socket(buf);
    }
}

impl Handler<CerrarConexion> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, _msg: CerrarConexion, ctx: &mut Context<Self>) {
        println!(
            "[{}] Cerrando conexión con estación {} y deteniendo actor EstacionCercana",
            self.estacion_local_id, self.estacion_id
        );

        // SOLUCIÓN: Dropear el sender ORIGINAL, no un clone
        // Creamos un channel dummy para reemplazar el original y poder dropearlo
        let (dummy_tx, _dummy_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let sender_original = std::mem::replace(&mut self.socket_estacion_cercana, dummy_tx);
        // Ahora dropeamos el sender ORIGINAL, lo que cierra el channel
        drop(sender_original);

        // Abortar el task de lectura para cerrar el reader explícitamente
        if let Some(reader_task) = self.reader_task.take() {
            println!("[{}] Abortando task de lectura para cerrar reader y notificar desconexión a estación {}", 
                     self.estacion_local_id, self.estacion_id);
            reader_task.abort(); // Abortar el task, lo que dropeará el reader y cerrará el socket TCP
        }

        // Detener el actor
        ctx.stop();
    }
}

impl EstacionCercana {
    pub async fn new(
        estacion_id: usize,
        estacion_local: Addr<Estacion>,
        reader: OwnedReadHalf,
        mut writer: OwnedWriteHalf,
        estacion_local_id: usize,
    ) -> EstacionCercana {
        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();

        let id_clone = estacion_id;
        let estacion_local_clone = estacion_local.clone();

        // Task que posee el writer y serializa las escrituras
        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                println!("Enviamos mensaje al socket de la estacion {}", id_clone);
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("Error writing to socket: {}", e);
                    match deserialize_message(&buf) {
                        Ok(message) => match message {
                            MessageType::Eleccion(m) => {
                                estacion_local_clone.do_send(EnviarASiguiente {
                                    estacion_cercana_id: id_clone,
                                    msg: m.to_bytes(),
                                })
                            }
                            MessageType::NotificarLider(m) => {
                                estacion_local_clone.do_send(EnviarASiguiente {
                                    estacion_cercana_id: id_clone,
                                    msg: m.to_bytes(),
                                })
                            }
                            MessageType::InformarVenta(m) => {
                                estacion_local_clone.do_send(LiderCaido { mensaje: m })
                            }
                            MessageType::InformarVentasOffline(m) => {
                                estacion_local_clone.do_send(EnviarASiguiente {
                                    estacion_cercana_id: id_clone,
                                    msg: m.to_bytes(),
                                })
                            }
                            _ => println!("Mensaje no manejado en desconexión"),
                        },
                        Err(e) => eprintln!("(Procesar) Error deserializando: {}", e),
                    }

                    break;
                }
            }
            // aquí puedes notificar desconexión si es necesario
        });

        //let reader_task = EstacionCercana::read_from_socket(reader, estacion_local.clone(), estacion_id);

        EstacionCercana {
            estacion_id,
            estacion_local,
            socket_estacion_cercana: tx,
            estacion_local_id,
            reader_task: None,
            reader: Some(reader),
            desconectado: false,
        }
    }

    pub fn enviar_por_socket(&mut self, buf: Vec<u8>) {
        if self.socket_estacion_cercana.send(buf.clone()).is_err() || self.desconectado {
            println!(
                "Error al enviar mensaje al socket de la estación: {}",
                self.estacion_id
            );
            /*self.estacion_local.do_send(EstacionDesconectada {
                estacion_id: self.estacion_id,
                mensaje: buf,
            });*/

            match deserialize_message(&buf) {
                Ok(message) => match message {
                    MessageType::Eleccion(m) => self.estacion_local.do_send(EnviarASiguiente {
                        estacion_cercana_id: self.estacion_id,
                        msg: m.to_bytes(),
                    }),
                    MessageType::NotificarLider(m) => {
                        self.estacion_local.do_send(EnviarASiguiente {
                            estacion_cercana_id: self.estacion_id,
                            msg: m.to_bytes(),
                        })
                    }
                    MessageType::InformarVenta(m) => {
                        self.estacion_local.do_send(LiderCaido { mensaje: m })
                    }
                    MessageType::InformarVentasOffline(m) => {
                        self.estacion_local.do_send(EnviarASiguiente {
                            estacion_cercana_id: self.estacion_id,
                            msg: m.to_bytes(),
                        })
                    }
                    _ => println!("Mensaje no manejado en desconexión"),
                },
                Err(e) => eprintln!("(Procesar) Error deserializando: {}", e),
            }
        } else {
            println!(
                "Enviamos mensaje al socket de la estación: {}",
                self.estacion_id
            );
        }
    }

    pub fn read_from_socket(
        mut reader: OwnedReadHalf,
        estacion_local: Addr<Estacion>,
        estacion_remota_id: usize,
        estacion: Addr<EstacionCercana>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match reader.read(&mut buf).await {
                    Ok(bytes) => {
                        if bytes == 0 {
                            println!(
                                "[{}] Reader detectó fin de conexión (0 bytes)",
                                estacion_remota_id
                            );
                            estacion.do_send(EstacionCercanaCerroConexion);
                            return;
                        }

                        println!(
                            "Recibimos mensaje del socket de la estacion {}",
                            estacion_remota_id
                        );

                        estacion_local.do_send(ProcesarMensaje {
                            bytes: buf[..bytes].to_vec(),
                            _estacion_remota: estacion_remota_id,
                        });
                    }
                    Err(e) => {
                        println!("[{}] Error leyendo del socket: {:?}", estacion_remota_id, e);
                        return;
                    }
                }
            }
        })
    }
}
