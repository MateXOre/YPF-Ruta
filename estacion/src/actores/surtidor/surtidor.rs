use crate::actores::estacion::Estacion;
use crate::actores::surtidor::messages::{CargarCombustible, Detenerme};
use actix::{Actor, Addr, AsyncContext, Context};
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use util::structs::venta::{EstadoVenta, Venta};
use util::{log_debug, log_error, log_info, log_warning};

pub struct Surtidor {
    pub(crate) id: usize,
    pub(crate) estacion: Addr<Estacion>,
    pub(crate) estacion_id: usize,
    pub(crate) reader: Option<OwnedReadHalf>,
    pub(crate) writer_tx: UnboundedSender<Vec<u8>>,

    pub(crate) logger: Sender<Vec<u8>>,
}

impl Surtidor {
    pub fn new(
        id: usize,
        estacion: Addr<Estacion>,
        cliente: TcpStream,
        estacion_id: usize,
        logger: Sender<Vec<u8>>,
    ) -> Self {
        let (reader, writer) = cliente.into_split();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

        let log = logger.clone();
        tokio::spawn(async move {
            let mut writer = writer;
            while let Some(buf) = rx.recv().await {
                if buf.is_empty() {
                    break;
                }
                if let Err(e) = writer.write_all(&buf).await {
                    log_error!(log, "Error al escribir: {:?}", e);
                    break;
                }
            }
        });

        Surtidor {
            id,
            estacion,
            estacion_id,
            reader: Some(reader),
            writer_tx: tx,
            logger,
        }
    }
}

impl Actor for Surtidor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log_info!(
            self.logger,
            "[{}] Surtidor conectado a la estación",
            self.estacion_id
        );

        let _ = self.writer_tx.send(b"Ingrese tarjeta=monto\n".to_vec());

        let mut reader = match self.reader.take() {
            Some(r) => r,
            None => {
                log_error!(
                    self.logger,
                    "[{}] ({}) Error: reader no disponible al iniciar el surtidor",
                    self.estacion_id,
                    self.id
                );
                return;
            }
        };
        let id_surtidor = self.id;
        let estacion_id = self.estacion_id;
        let surtidor_addr = ctx.address();
        let writer_tx = self.writer_tx.clone();

        let logger = self.logger.clone();
        actix_rt::spawn(async move {
            let mut buffer = [0u8; 128];
            let mut continuar = true;

            while continuar {
                let result = reader.read(&mut buffer).await;

                match result {
                    Ok(n) if n > 0 => {
                        let mensaje = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        let partes: Vec<&str> = mensaje.split('=').collect();

                        if partes.len() == 2 {
                            let monto: f32 = match partes[1].parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    log_warning!(
                                        logger,
                                        "[{}] ({}) Monto inválido: {}",
                                        estacion_id,
                                        id_surtidor,
                                        partes[1]
                                    );
                                    continue;
                                }
                            };
                            let id: usize = match partes[0].parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    log_warning!(
                                        logger,
                                        "[{}] ({}) Tarjeta inválida: {}",
                                        estacion_id,
                                        id_surtidor,
                                        partes[0]
                                    );
                                    continue;
                                }
                            };

                            let timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_millis())
                                .unwrap_or_else(|e| {
                                    log_error!(
                                        logger,
                                        "[{}] ({}) SystemTime antes del UNIX_EPOCH: {:?}",
                                        estacion_id,
                                        id_surtidor,
                                        e
                                    );
                                    0u128
                                });

                            let id_str = format!("{:04}{:010}", estacion_id, timestamp);
                            let id_venta = match id_str.parse::<usize>() {
                                Ok(v) => v,
                                Err(e) => {
                                    log_error!(
                                        logger,
                                        "[{}] ({}) Error al parsear id_venta '{}': {:?}",
                                        estacion_id,
                                        id_surtidor,
                                        id_str,
                                        e
                                    );
                                    0
                                }
                            };

                            log_debug!(logger, "[{}] ({}) ID de venta: {}", estacion_id, id_surtidor, id_venta);
                            let venta = Venta {
                                id_venta,
                                id_tarjeta: id,
                                monto,
                                offline: false,
                                estado: EstadoVenta::Pendiente,
                                id_estacion: estacion_id,
                            };

                            surtidor_addr.do_send(CargarCombustible { venta });
                            continuar = false;
                        } else {
                            let _ =
                                writer_tx.send(b"Formato invalido, use tarjeta=monto\n".to_vec());
                            log_warning!(logger, "[{}] ({}) Formato inválido: {}", estacion_id, id_surtidor, mensaje);
                        }
                    }
                    Ok(_) => {
                        log_info!(logger, "[{}] ({}) Cliente desconectado", estacion_id, id_surtidor);
                        surtidor_addr.do_send(Detenerme);
                        continuar = false;
                    }
                    Err(e) => {
                        log_error!(logger, "[{}] ({}) Error al leer: {:?}", estacion_id, id_surtidor, e);
                        surtidor_addr.do_send(Detenerme);
                        continuar = false;
                    }
                }
            }
        });
    }
}
