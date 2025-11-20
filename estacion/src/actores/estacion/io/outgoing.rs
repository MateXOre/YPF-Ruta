use actix::prelude::*;
use tokio::net::TcpStream;
//use tokio::io::AsyncWriteExt;
//use tokio_util::codec::{FramedRead, LinesCodec};
//use futures::StreamExt;

use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use crate::actores::estacion::io::procesar_mensaje;
use crate::actores::estacion_cercana::EstacionCercana;

pub async fn handle_stream_outgoing(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
    id_destino: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let peer_addr = stream.peer_addr()?;

    println!("Vamos a intentar establecer conexión de {}", id_destino);
    let server_addr_clone = server_addr.clone();
    let estacion = EstacionCercana::new(id_destino, server_addr_clone, stream).await;

    let estacion_addr = estacion.start();
    server_addr.do_send(AgregarEstacion {
        estacion: estacion_addr,
        estacion_id: id_destino,
    });

    Ok(())
}