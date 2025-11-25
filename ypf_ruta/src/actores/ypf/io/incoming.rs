use crate::actores::ypf::messages::*;
use crate::actores::ypf::{EmpresaConectada, YpfRuta};
use actix::prelude::*;
use std::sync::mpsc::Sender;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use util::{log_debug, log_error, log_info};

pub async fn handle_stream_incoming(
    stream: TcpStream,
    _id: usize,
    server_addr: Addr<YpfRuta>,
    logger: Sender<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut reader, writer) = stream.into_split();

    let mut buf = [0u8; 128];
    let bytes = reader.read(&mut buf).await?;

    let mensaje = buf[..bytes].to_vec();
    let opcode = mensaje[0];

    let id_remoto = match opcode {
        OPCODE_IDENTIFICAR_EMPRESA => match IdentificarEmpresa::from_bytes(&mensaje) {
            Ok(msg) => msg.id,
            Err(e) => {
                log_error!(logger, "Error deserializando IdentificarEmpresa: {}", e);
                return Ok(());
            }
        },
        _ => {
            log_error!(
                logger,
                "Opcode desconocido recibido al identificar empresa: 0x{:02x}",
                opcode
            );
            return Ok(());
        }
    };
    log_info!(logger, "Empresa remota identificada como {}", id_remoto);

    let logger_clone = logger.clone();

    let server_addr_clone = server_addr.clone();
    let empresa =
        EmpresaConectada::new(server_addr_clone, reader, writer, id_remoto, logger_clone).await;

    let empresa_addr = empresa.start();
    server_addr.do_send(AgregarEmpresa {
        empresa: empresa_addr,
        empresa_id: id_remoto,
    });

    Ok(())
}
