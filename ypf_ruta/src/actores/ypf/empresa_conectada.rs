use std::sync::mpsc::Sender;
use actix::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use util::{log_debug, log_error};
use crate::actores::ypf::YpfRuta;
use crate::actores::ypf::messages::ProcesarMensajeEmpresa;



#[derive(Message)]
#[rtype(result = "()")]
pub struct Enviar {
    pub bytes: Vec<u8>,
}

impl Handler<Enviar> for EmpresaConectada {
    type Result = ();

    fn handle(&mut self, msg: Enviar, _ctx: &mut Context<Self>) {
        self.logger.send(format!("Enviando mensaje a la empresa conectada {}", self.empresa_id).into_bytes()).unwrap_or(());
        let buf = msg.bytes.clone();
        let logger_clone = self.logger.clone();
        self.enviar_por_socket(buf, logger_clone);
    }
}



pub struct EmpresaConectada {
    pub ypf_ruta: Addr<YpfRuta>,
    pub socket_empresa_conectada: UnboundedSender<Vec<u8>>,
    reader: Option<OwnedReadHalf>,
    pub empresa_id: usize,
    logger: Sender<Vec<u8>>,
}



impl EmpresaConectada {
    pub async fn new(ypf_ruta: Addr<YpfRuta>, reader: OwnedReadHalf, mut writer: OwnedWriteHalf, empresa_id: usize, logger: Sender<Vec<u8>>) -> Self {
        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();

        let logger_clone = logger.clone();
        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                if let Err(e) = writer.write_all(&buf).await {
                    log_error!(
                        logger_clone,
                        "Error writing to socket: {}",
                        e
                    );
                    break;
                }
            }
        });

        Self {
            ypf_ruta,
            socket_empresa_conectada: tx,
            reader: Some(reader),
            empresa_id,
            logger
        }
    }

    pub fn read_from_socket(
        mut reader: OwnedReadHalf,
        ypf_ruta: Addr<YpfRuta>,
        logger: Sender<Vec<u8>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf = vec![0; 8192];

            loop {
                match reader.read(&mut buf).await {
                    Ok(bytes) => {
                        if bytes == 0 {
                            log_debug!(
                                logger,
                                "Reader detectó fin de conexión (0 bytes)"
                            );
                        }
                        ypf_ruta.do_send(ProcesarMensajeEmpresa {
                            bytes: buf[..bytes].to_vec(),
                        });
                    }
                    Err(e) => {
                        log_error!(
                                logger,
                                "Error leyendo del socket: {:?}",
                                e
                        );
                        return;
                    }
                }
            }
        })
    }


    pub fn enviar_por_socket(&mut self, buf: Vec<u8>, logger: Sender<Vec<u8>>) {
        if let Err(e) = self.socket_empresa_conectada.send(buf.clone()) {
            log_error!(
                logger,
                "Error al enviar mensaje al socket de empresa Conectada: {}",
                e
            );
        } else {
            log_debug!(
                logger,
                "Enviamos mensaje al socket de Empresa Conectada"
            );
        }
    }
}



impl Actor for EmpresaConectada {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Context<Self>) {
        let reader = self.reader.take().expect("reader debería estar");

        EmpresaConectada::read_from_socket(
            reader,
            self.ypf_ruta.clone(),
            self.logger.clone(),
        );
    }
}

