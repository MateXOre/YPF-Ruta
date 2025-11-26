use crate::actores::estacion::messages::InformarVenta;

use crate::actores::estacion::Eleccion;
use crate::actores::estacion::Estacion;
use crate::actores::estacion::NotificarLider;
use crate::actores::estacion::{
    deserialize_message, ConfirmarTransacciones, DesconexionDetectada, EnviarASiguiente,
    LiderCaido, MessageType, ProcesarMensaje,
};
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler, Message};
use std::sync::mpsc::Sender;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use util::log_debug;
use util::log_error;
use util::log_info;
use util::log_warning;

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

    pub logger: Sender<Vec<u8>>,
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
            self.logger.clone(),
        );
        self.reader_task = Some(reader_task);
    }
}

impl Handler<Enviar> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: Enviar, _ctx: &mut Context<Self>) {
        let buf = msg.bytes.clone();
        self.enviar_por_socket(buf);
    }
}

impl Handler<EstacionCercanaCerroConexion> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, _msg: EstacionCercanaCerroConexion, _ctx: &mut Context<Self>) {
        log_info!(
            self.logger,
            "[{}] Estación cercana {} cerró la conexión, marcando como desconectada",
            self.estacion_local_id,
            self.estacion_id
        );

        self.desconectado = true;

        self.estacion_local.do_send(DesconexionDetectada {
            estacion_id: self.estacion_id,
        });
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
        log_info!(
            self.logger,
            "[{}] Cerrando conexión con estación {} y deteniendo actor EstacionCercana",
            self.estacion_local_id,
            self.estacion_id
        );

        let (dummy_tx, _dummy_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let sender_original = std::mem::replace(&mut self.socket_estacion_cercana, dummy_tx);
        drop(sender_original);

        if let Some(reader_task) = self.reader_task.take() {
            log_info!(
                self.logger,
                "[{}] Abortando task de lectura para cerrar reader y notificar desconexión a estación {}",
                self.estacion_local_id, self.estacion_id
            );
            reader_task.abort();
        }

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
        logger: Sender<Vec<u8>>,
    ) -> EstacionCercana {
        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();

        let id_clone = estacion_id;
        let estacion_local_clone = estacion_local.clone();

        // Task que posee el writer y serializa las escrituras
        let logger_clone = logger.clone();
        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                let log = logger_clone.clone();
                log_debug!(
                    log,
                    "Enviando mensaje al socket de la estación {}",
                    id_clone
                );
                if let Err(e) = writer.write_all(&buf).await {
                    log_error!(log, "Error writing to socket: {}", e);
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
                            _ => log_warning!(log, "Mensaje no manejado en desconexión"),
                        },
                        Err(e) => log_error!(log, "(Procesar) Error deserializando: {}", e),
                    }

                    break;
                }
            }
        });

        EstacionCercana {
            estacion_id,
            estacion_local,
            socket_estacion_cercana: tx,
            estacion_local_id,
            reader_task: None,
            reader: Some(reader),
            desconectado: false,
            logger,
        }
    }

    pub fn enviar_por_socket(&mut self, buf: Vec<u8>) {
        if self.socket_estacion_cercana.send(buf.clone()).is_err() || self.desconectado {
            log_error!(
                self.logger,
                "Error al enviar mensaje al socket de la estación {}",
                self.estacion_id
            );

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
                    _ => log_warning!(self.logger, "Mensaje no manejado en desconexión"),
                },
                Err(e) => log_error!(self.logger, "(Procesar) Error deserializando: {}", e),
            }
        }
    }

    pub fn read_from_socket(
        mut reader: OwnedReadHalf,
        estacion_local: Addr<Estacion>,
        estacion_remota_id: usize,
        estacion: Addr<EstacionCercana>,
        logger: Sender<Vec<u8>>,
    ) -> tokio::task::JoinHandle<()> {
        let log = logger.clone();
        tokio::spawn(async move {
            let mut buf = vec![0; 8192];

            loop {
                match reader.read(&mut buf).await {
                    Ok(bytes) => {
                        if bytes == 0 {
                            log_warning!(
                                log,
                                "[{}] Reader detectó fin de conexión (0 bytes)",
                                estacion_remota_id
                            );
                            estacion.do_send(EstacionCercanaCerroConexion);
                            return;
                        }

                        log_info!(
                            log,
                            "Recibimos mensaje del socket de la estacion {}",
                            estacion_remota_id
                        );

                        estacion_local.do_send(ProcesarMensaje {
                            bytes: buf[..bytes].to_vec(),
                            _estacion_remota: estacion_remota_id,
                        });
                    }
                    Err(e) => {
                        log_error!(
                            log,
                            "[{}] Error leyendo del socket: {:?}",
                            estacion_remota_id,
                            e
                        );
                        return;
                    }
                }
            }
        })
    }
}
