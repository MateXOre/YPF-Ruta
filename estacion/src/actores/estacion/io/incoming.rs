use actix::prelude::*;
use std::sync::mpsc::Sender;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use util::log_info;
use util::log_warning;

use crate::actores::estacion::messages::*;
use crate::actores::estacion::Estacion;
use crate::actores::estacion_cercana::EstacionCercana;

pub async fn handle_stream_incoming(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
    logger: Sender<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _peer_addr = stream.peer_addr()?;

    let (mut reader, writer) = stream.into_split();

    let mut buf = [0u8; 128];
    let bytes = reader.read(&mut buf).await?;

    let mensaje = buf[..bytes].to_vec();
    let opcode = mensaje[0];

    let mensaje_deserializado = match opcode {
        OPCODE_IDENTIFICAR => {
            IdentificarEstacion::from_bytes(&mensaje).map(MessageType::IdentificarEstacion)
        }
        _ => Err(format!(
            "Opcode desconocido: 0x{:02x}, solo se conoce {}",
            opcode, OPCODE_IDENTIFICAR
        )),
    };

    let id_remoto = match mensaje_deserializado {
        Ok(MessageType::IdentificarEstacion(msg)) => msg.id,
        _ => {
            log_warning!(logger, "Mensaje inesperado recibido");
            return Ok(());
        }
    };
    log_info!(
        logger,
        "[{}] estación remota identificada como {}",
        id,
        id_remoto
    );
    let server_addr_clone = server_addr.clone();
    let estacion =
        EstacionCercana::new(id_remoto, server_addr_clone, reader, writer, id, logger).await;

    let estacion_addr = estacion.start();
    server_addr.do_send(AgregarEstacion {
        estacion: estacion_addr,
        estacion_id: id_remoto,
    });

    Ok(())
}
