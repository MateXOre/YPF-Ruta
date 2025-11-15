use actix::prelude::*;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{FramedRead, LinesCodec};
use futures::StreamExt;

use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use crate::actores::estacion::io::procesar_mensaje;
use crate::actores::estacion_cercana::EstacionCercana;

pub async fn handle_stream_outgoing(
    stream: TcpStream,
    id: usize,
    server_addr: Addr<Estacion>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let peer_addr = stream.peer_addr()?;

    let (r, mut w) = stream.into_split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    let conn = EstacionCercana {
        estacion_id: format!("{}(outgoing)", id),
        tx: tx.clone(),
    }
        .start();

    server_addr.do_send(AgregarEstacion {
        peer: conn.clone(),
        peer_addr,
    });

    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if w.write_all(format!("{}\n", line).as_bytes()).await.is_err() {
                break;
            }
        }
    });

    let mut reader = FramedRead::new(r, LinesCodec::new());

    while let Some(res) = reader.next().await {
        match res {
            Ok(line) => {
                procesar_mensaje(&line, &server_addr, &conn);
            }
            Err(e) => {
                eprintln!("Error lectura saliente en {}: {:?}", id, e);
                break;
            }
        }
    }

    Ok(())
}