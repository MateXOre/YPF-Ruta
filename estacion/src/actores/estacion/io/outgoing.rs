use actix::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::actores::estacion::messages::*;
use crate::actores::estacion::Estacion;
use crate::actores::estacion_cercana::EstacionCercana;

pub async fn handle_stream_outgoing(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
    id_destino: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _peer_addr = stream.peer_addr()?;

    println!("Vamos a intentar establecer conexión de {}", id_destino);
    let server_addr_clone = server_addr.clone();

    let (reader, mut writer) = stream.into_split();

    let msg = crate::actores::estacion::IdentificarEstacion { id }.to_bytes();

    if let Err(e) = writer.write_all(&msg).await {
        eprintln!("Error writing to socket: {}", e);
    }

    let estacion = EstacionCercana::new(id_destino, server_addr_clone, reader, writer, id).await;
    println!("Se crea estacion cercana para {}", id_destino);

    let estacion_addr = estacion.start();
    server_addr.do_send(AgregarEstacion {
        estacion: estacion_addr,
        estacion_id: id_destino,
    });

    Ok(())
}
