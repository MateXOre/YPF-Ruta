use crate::actores::estacion::messages::Solicitud;
pub(crate) use crate::actores::estacion::messages::ValidarVentas;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::prelude::*;
use serde_json;
use std::sync::mpsc::Sender;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use util::{log_info, log_warning};

pub struct Estacion {
    pub(crate) socket: Option<OwnedWriteHalf>,
    ypf_addr: Addr<YpfRuta>,
    ventas_iniciales: Option<Solicitud>,
    pub(crate) logger: Sender<Vec<u8>>,
}

impl Estacion {
    pub async fn new(
        socket: TcpStream,
        ypf_addr: Addr<YpfRuta>,
        logger: Sender<Vec<u8>>,
    ) -> Result<Estacion, ()> {
        let (mut reader, writer) = socket.into_split();

        let mut len_bytes = [0u8; 4];
        if reader.read_exact(&mut len_bytes).await.is_err() {
            return Err(());
        };
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        if reader.read_exact(&mut buffer).await.is_err() {
            return Err(());
        };

        let solicitud: Solicitud = if let Ok(s) = serde_json::from_slice(&buffer) {
            s
        } else {
            return Err(());
        };

        Ok(Estacion {
            socket: Some(writer),
            ypf_addr,
            ventas_iniciales: Some(solicitud),
            logger,
        })
    }
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log_info!(self.logger, "Estacion actor iniciado.");
        if let Some(ventas) = self.ventas_iniciales.take() {
            let from = ctx.address();
            self.ypf_addr.do_send(ValidarVentas { ventas, from });
            log_info!(self.logger, "Estacion: envié ValidarVentas a YpfRuta.");
        } else {
            log_warning!(
                self.logger,
                "Estacion: no se recibieron ventas iniciales al conectar."
            );
        }
    }
}
