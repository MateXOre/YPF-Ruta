use actix::{Actor, Context, Handler, Message, Addr};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use crate::actores::estacion::ConfirmarTransacciones;
use crate::actores::estacion::messages::InformarVenta;
use crate::actores::estacion::messages::Reenviar;
use crate::actores::estacion::Estacion;
use std::str::FromStr;

// ===== Mensajes =====
/*
Eleccion
NotificarLider
InformarVentasOffline
ValidarVentas
*/

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConectarEstacion(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct RespuestaConexion(pub String);

pub struct EstacionCercana {
    pub estacion_id: usize,
    pub estacion_local: Addr<Estacion>,
    pub socket_estacion_cercana: OwnedWriteHalf
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
        self.socket_estacion_cercana.write_all(&buf);
        println!("Enviamos mensaje al socket de la estación: {}", self.estacion_id);
    }
}

impl EstacionCercana {
    pub async fn new(estacion_id: usize, estacion_local: Addr<Estacion>, estacion_cercana_socket: TcpStream) -> EstacionCercana {
        let addr = estacion_local.clone();
        let (mut reader, writer) = estacion_cercana_socket.into_split();

        let estacion_cercana = EstacionCercana {
            estacion_id,
            estacion_local,
            socket_estacion_cercana: writer
        };

        EstacionCercana::read_from_socket(reader, addr).await;

        estacion_cercana
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
                estacion_local.do_send(message)
            }
        });
    }
    
}
