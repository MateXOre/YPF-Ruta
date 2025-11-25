use actix::prelude::*;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::actores::ypf::messages::*;
use crate::actores::ypf::{YpfRuta, EmpresaConectada};


pub async fn handle_stream_incoming(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<YpfRuta>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut reader, writer) = stream.into_split();

    let mut buf = [0u8; 128];
    println!("HANDLE STREAM INCOMING, esperando identificacion");
    let bytes = reader.read(&mut buf).await?;

    let mensaje = buf[..bytes].to_vec();
    let opcode = mensaje[0];

    let id_remoto = match opcode {
        OPCODE_IDENTIFICAR_EMPRESA => {
            match IdentificarEmpresa::from_bytes(&mensaje) {
                Ok(msg) => msg.id,
                Err(e) => {
                    println!("Error deserializando IdentificarEmpresa: {}", e);
                    return Ok(());
                }
            }
        }
        _ => {
            println!(
                "Opcode desconocido: 0x{:02x}, solo se conoce {}",
                opcode, OPCODE_IDENTIFICAR_EMPRESA
            );
            return Ok(());
        }
    };
    println!("[{}] empresa remota identificada como {}", id, id_remoto);

    let server_addr_clone = server_addr.clone();
    let empresa = EmpresaConectada::new(server_addr_clone, reader, writer, id_remoto).await;

    let empresa_addr = empresa.start();
    server_addr.do_send(AgregarEmpresa {
        empresa: empresa_addr,
        empresa_id: id_remoto,
    });

    Ok(())
}