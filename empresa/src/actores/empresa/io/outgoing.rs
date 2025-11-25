use actix::prelude::*;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::actores::empresa::messages::*;
use crate::actores::empresa::Empresa;
use crate::actores::ypf_ruta::ypf_ruta_actor::YpfRuta;

pub async fn handle_stream_outgoing(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Empresa>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _peer_addr = stream.peer_addr()?;

    let server_addr_clone = server_addr.clone();

    let (reader, mut writer) = stream.into_split();

    let msg = IdentificarEmpresa { id }.to_bytes();

    if let Err(e) = writer.write_all(&msg).await {
        eprintln!("Error writing to socket: {}", e);
    }
    let ypf_ruta_addr = YpfRuta::new(server_addr_clone, reader, writer).start();
    println!("Se crea YpfRuta");

    server_addr.do_send(ConectadoAypfRuta {
        addr: ypf_ruta_addr,
    });

    Ok(())
}
