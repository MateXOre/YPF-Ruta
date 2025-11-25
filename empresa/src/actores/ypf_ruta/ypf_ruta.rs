use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler, Message};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use crate::actores::empresa::{Empresa};

use crate::actores::empresa::{
    messages::{ProcesarMensajeSocket},
};

pub struct YpfRuta {
    pub empresa_local: Addr<Empresa>,
    pub socket_ypf_ruta: UnboundedSender<Vec<u8>>,
    reader: Option<OwnedReadHalf>
}

impl YpfRuta {
    pub fn new(empresa_local: Addr<Empresa>, reader: OwnedReadHalf, mut writer: OwnedWriteHalf) -> Self {
        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();

        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                println!("Enviamos mensaje al socket de YPF Ruta");
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                }
            }
        });

        Self {
            empresa_local,
            socket_ypf_ruta: tx,
            reader: Some(reader),
        }
    }
}

impl Actor for YpfRuta {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let ypf_ruta_addr = ctx.address();

        let reader = self.reader.take().expect("reader debería estar");

        YpfRuta::read_from_socket(
            reader,
            self.empresa_local.clone(),
        );
    }
}

impl YpfRuta {
    pub fn read_from_socket(
        mut reader: OwnedReadHalf,
        empresa_local: Addr<Empresa>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf = vec![0; 8192];

            loop {
                match reader.read(&mut buf).await {
                    Ok(bytes) => {
                        if bytes == 0 {
                            println!(
                                "Reader detectó fin de conexión (0 bytes)",
                            );
                            return;
                        }
                        println!(
                            "Recibimos mensaje del socket: {} bytes",
                            bytes
                        );

                        empresa_local.do_send(ProcesarMensajeSocket {
                            bytes: buf[..bytes].to_vec(),
                        });
                    }
                    Err(e) => {
                        println!("Error leyendo del socket: {:?}", e);
                        return;
                    }
                }
            }
        })
    }


    pub fn enviar_por_socket(&mut self, buf: Vec<u8>) {
        if let Err(e) = self.socket_ypf_ruta.send(buf.clone()) {
            println!(
                "Error al enviar mensaje al socket de YPF Ruta: {}",
                e
            );
        } else {
            println!(
                "Enviamos mensaje al socket de YPF Ruta",
            )
        }
    }
}