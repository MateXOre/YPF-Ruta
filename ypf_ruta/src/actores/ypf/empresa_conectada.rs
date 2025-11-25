use actix::prelude::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
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
        println!(
            "Enviando mensaje a la empresa conectada {}",
            self.empresa_id
        );
        let buf = msg.bytes.clone();
        self.enviar_por_socket(buf);
    }
}



pub struct EmpresaConectada {
    pub ypf_ruta: Addr<YpfRuta>,
    pub socket_empresa_conectada: UnboundedSender<Vec<u8>>,
    reader: Option<OwnedReadHalf>,
    pub empresa_id: usize,
}



impl EmpresaConectada {
    pub async fn new(ypf_ruta: Addr<YpfRuta>, reader: OwnedReadHalf, mut writer: OwnedWriteHalf, empresa_id: usize) -> Self {
        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();

        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                println!("Enviamos mensaje al socket de Empresa Conectada");
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                }
            }
        });

        Self {
            ypf_ruta,
            socket_empresa_conectada: tx,
            reader: Some(reader),
            empresa_id,
        }
    }

    pub fn read_from_socket(
        mut reader: OwnedReadHalf,
        ypf_ruta: Addr<YpfRuta>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf = vec![0; 8192];

            loop {
                match reader.read(&mut buf).await {
                    Ok(bytes) => {
                        println!(
                            "Recibimos mensaje del socket: {} bytes",
                            bytes
                        );

                        ypf_ruta.do_send(ProcesarMensajeEmpresa {
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
        if let Err(e) = self.socket_empresa_conectada.send(buf.clone()) {
            println!(
                "Error al enviar mensaje al socket de Empresa Conectada: {}",
                e
            );
        } else {
            println!(
                "Enviamos mensaje al socket de Empresa Conectada",
            )
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
        );
    }
}

