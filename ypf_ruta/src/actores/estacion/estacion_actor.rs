use actix::prelude::*;
use tokio::net::tcp::OwnedWriteHalf;
use crate::actores::estacion::messages::Solicitud;
use crate::actores::ypf::ypf_actor::YpfRuta;
use serde_json;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt};
pub(crate) use crate::actores::estacion::messages::ValidarVentas;


pub struct Estacion {
    pub(crate) socket: Option<OwnedWriteHalf>,
    ypf_addr: Addr<YpfRuta>,
    ventas_iniciales: Option<Solicitud>,
}

impl Estacion {
    pub async fn new(socket: TcpStream, ypf_addr: Addr<YpfRuta>) -> Result<Estacion, ()> {
        let (mut reader, writer) = socket.into_split();
        
        // Leer el tamaño (4 bytes)
        let mut len_bytes = [0u8; 4];
        if reader.read_exact(&mut len_bytes).await.is_err() {
            return Err(())
        };
        let len = u32::from_be_bytes(len_bytes) as usize;
        
        // Leer los datos
        let mut buffer = vec![0u8; len];
        if reader.read_exact(&mut buffer).await.is_err() {
            return Err(())
        };

        let solicitud: Solicitud = if let Ok(s) = serde_json::from_slice(&buffer) {
            s
        } else {
            return Err(())
        };

        Ok(Estacion {
            socket: Some(writer),
            ypf_addr,
            ventas_iniciales: Some(solicitud),
        })
    }
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Estacion actor iniciado.");
        if let Some(ventas) = self.ventas_iniciales.take() {
            let from = ctx.address();
            self.ypf_addr.do_send(ValidarVentas { ventas, from });
            println!("Estacion: envié ValidarVentas a YpfRuta.");
        } else {
            println!("Estacion: no se recibieron ventas iniciales al conectar.");
        }
    }
}
