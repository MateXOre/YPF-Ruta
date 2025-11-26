use actix::prelude::*;
use std::sync::mpsc::Sender;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use util::log_error;
use util::log_info;

use crate::actores::estacion::messages::*;
use crate::actores::estacion::Estacion;
use crate::actores::estacion_cercana::EstacionCercana;

pub async fn handle_stream_outgoing(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
    id_destino: usize,
    logger: Sender<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _peer_addr = stream.peer_addr()?;

    let server_addr_clone = server_addr.clone();

    let (reader, mut writer) = stream.into_split();

    let msg = crate::actores::estacion::IdentificarEstacion { id }.to_bytes();

    if let Err(e) = writer.write_all(&msg).await {
        log_error!(logger, "Error writing to socket: {}", e);
    }

    let estacion = EstacionCercana::new(
        id_destino,
        server_addr_clone,
        reader,
        writer,
        id,
        logger.clone(),
    )
    .await;
    log_info!(logger, "Se crea estacion cercana para {}", id_destino);

    let estacion_addr = estacion.start();
    server_addr.do_send(AgregarEstacion {
        estacion: estacion_addr,
        estacion_id: id_destino,
    });

    Ok(())
}
