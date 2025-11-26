use crate::actores::estacion::{Estacion, TransaccionesPorEstacion};
use actix::{Actor, Addr, Context};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use util::{log_error, log_info};

const YPF_ADDRS: [&str; 3] = ["127.0.0.1:18080", "127.0.0.1:18081", "127.0.0.1:18082"];

pub struct Ypf {
    writer: Option<OwnedWriteHalf>,
    reader: Option<OwnedReadHalf>,
    estacion_addr: Addr<Estacion>,
    logger: Sender<Vec<u8>>,
}

impl Ypf {
    pub async fn new(estacion_addr: Addr<Estacion>, logger: Sender<Vec<u8>>) -> Result<Self, ()> {
        for addr in YPF_ADDRS.iter() {
            log_info!(logger, "Intentando conectarnos a YPF en {}", addr);
            if let Ok(socket) = tokio::net::TcpStream::connect(addr).await {
                log_info!(logger, "Conectado a YPF en {}", addr);

                let (reader, writer) = socket.into_split();

                return Ok(Ypf {
                    writer: Some(writer),
                    reader: Some(reader),
                    estacion_addr,
                    logger,
                });
            } else {
                log_error!(logger, "No se pudo conectar a YPF en {}", addr);
            }
        }

        Err(())
    }

    pub fn enviar_por_socket(&mut self, bytes: Vec<u8>) {
        let mut writer_opt = if let Some(w) = self.writer.take() {
            w
        } else {
            log_error!(
                self.logger,
                "No hay writer disponible para enviar datos a YPF."
            );
            return;
        };
        let logger = self.logger.clone();
        tokio::spawn(async move {
            if let Err(e) = writer_opt.write_all(&bytes).await {
                log_error!(logger, "Error enviando datos a YPF: {}", e);
            }
        });
    }

    pub fn leer_de_socket(&mut self) {
        let reader = if let Some(r) = self.reader.take() {
            r
        } else {
            log_error!(
                self.logger,
                "No hay reader disponible para leer datos de YPF."
            );
            return;
        };
        let estacion = self.estacion_addr.clone();

        let logger = self.logger.clone();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = Vec::new();

            match buf_reader.read_until(b'\n', &mut line).await {
                Ok(0) => {
                    log_error!(logger, "Conexión cerrada por YPF.");
                }
                Ok(_) => {
                    if line.last() == Some(&b'\n') {
                        line.pop();
                    }

                    let transacciones: HashMap<usize, HashMap<usize, Vec<(usize, bool)>>> =
                        match serde_json::from_slice(&line) {
                            Ok(data) => data,
                            Err(e) => {
                                log_error!(logger, "Error deserializando datos de YPF: {}", e);
                                log_error!(
                                    logger,
                                    "Datos recibidos: {:?}",
                                    String::from_utf8_lossy(&line)
                                );
                                return;
                            }
                        };
                    estacion.do_send(TransaccionesPorEstacion { transacciones });
                }
                Err(e) => {
                    log_error!(logger, "Error leyendo de YPF: {}", e);
                }
            }
        });
    }
}

impl Actor for Ypf {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log_info!(self.logger, "Actor Ypf iniciado.");
        self.leer_de_socket();
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log_info!(self.logger, "Actor Ypf detenido.");
    }
}
