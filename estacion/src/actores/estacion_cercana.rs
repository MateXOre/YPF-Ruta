use crate::actores::estacion::messages::EstacionDesconectada;
use crate::actores::estacion::messages::InformarVenta;
use crate::actores::estacion::messages::Reenviar;
use crate::actores::estacion::ConfirmarTransacciones;
use crate::actores::estacion::Eleccion;
use crate::actores::estacion::Estacion;
use crate::actores::estacion::NotificarLider;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, WrapFuture};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

// ===== Mensajes =====

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConectarEstacion(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct RespuestaConexion(pub String);

pub struct EstacionCercana {
    pub estacion_id: usize,
    pub estacion_local: Addr<Estacion>,
    pub socket_estacion_cercana: UnboundedSender<Vec<u8>>,
}

impl Actor for EstacionCercana {
    type Context = Context<Self>;
}

pub fn serialize(msg: Reenviar) -> Vec<u8> {
    msg.0.as_bytes().to_vec()
}

pub fn deserialize(buffer: Vec<u8>) -> Reenviar {
    Reenviar(String::from_utf8(buffer).unwrap())
}

impl Handler<Reenviar> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: Reenviar, _ctx: &mut Context<Self>) {
        let buf = serialize(msg);
        self.enviar_por_socket(buf);
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

impl EstacionCercana {
    pub async fn new(
        estacion_id: usize,
        estacion_local: Addr<Estacion>,
        estacion_cercana_socket: TcpStream,
    ) -> EstacionCercana {
        let addr = estacion_local.clone();
        let (reader, mut writer) = estacion_cercana_socket.into_split();

        let (tx, mut rx) = unbounded_channel::<Vec<u8>>();

        // Task que posee el writer y serializa las escrituras
        tokio::spawn(async move {
            while let Some(buf) = rx.recv().await {
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                }
            }
            // aquí puedes notificar desconexión si es necesario
        });

        let estacion_cercana = EstacionCercana {
            estacion_id,
            estacion_local,
            socket_estacion_cercana: tx,
        };

        EstacionCercana::read_from_socket(reader, addr).await;

        estacion_cercana
    }

    pub fn enviar_por_socket(&mut self, buf: Vec<u8>) {
        if self.socket_estacion_cercana.send(buf.clone()).is_err() {
            println!(
                "Error al enviar mensaje al socket de la estación: {}",
                self.estacion_id
            );
            self.estacion_local.do_send(EstacionDesconectada {
                estacion_id: self.estacion_id,
                mensaje: buf,
            });
        } else {
            println!(
                "Enviamos mensaje al socket de la estación: {}",
                self.estacion_id
            );
        }
    }

    pub async fn read_from_socket(mut reader: OwnedReadHalf, estacion_local: Addr<Estacion>) {
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                let bytes = reader
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if bytes == 0 {
                    return;
                }

                println!("Recibimos mensaje del socket");
                let message = deserialize(buf.clone());
                println!("Mensaje {}", message.0);
                estacion_local.do_send(message)
            }
        });
    }
}
