use actix::prelude::*;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::{FramedRead, LinesCodec};
use futures::StreamExt;


use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use crate::actores::estacion::io::procesar_mensaje;
use crate::actores::estacion_cercana::EstacionCercana;

pub async fn handle_stream_incoming(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let peer_addr = stream.peer_addr()?;

    let (mut reader, writer) = stream.into_split();

    let mut buf = [0u8; 128];
    println!("HANDLE STREAM INCOMING, esperando identificacion");
    let bytes = reader.read(&mut buf).await?;

    let mensaje = buf[..bytes].to_vec();
    let opcode = mensaje[0];



     let mensaje_deserializado = match opcode {
        OPCODE_IDENTIFICAR => IdentificarEstacion::from_bytes(&mensaje).map(MessageType::IdentificarEstacion),
        _ => Err(format!("Opcode desconocido: 0x{:02x}, solo se conoce {}", opcode, OPCODE_IDENTIFICAR)),
    };

    let id_remoto = match mensaje_deserializado {
        Ok(MessageType::IdentificarEstacion(msg)) =>
            msg.id,
        _ => {
            println!("Mensaje inesperado recibido");
            return Ok(());
        }
    };
    println!(
        "[{}] estación remota identificada como {}",
        id, id_remoto
    );

    println!("Vamos a intentar recibir conexión de {}", id_remoto);
    let server_addr_clone = server_addr.clone();
    let estacion = EstacionCercana::new(id_remoto, server_addr_clone, reader, writer, id).await;

    let estacion_addr = estacion.start();
    server_addr.do_send(AgregarEstacion {
        estacion: estacion_addr,
        estacion_id: id_remoto,
    });

    Ok(())
}