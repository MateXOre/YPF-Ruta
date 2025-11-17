use std::io::Read;

use actix::{Actor, Addr, AsyncContext, Context, WrapFuture};
use tokio::io::{AsyncReadExt};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use util::structs::venta::{EstadoVenta, Venta};
use crate::actores::estacion::Estacion;
use crate::actores::surtidor::messages::InformarVenta;

pub struct Surtidor {
    pub(crate) id: usize,
    pub(crate) estacion: Addr<Estacion>,
    pub(crate) estacion_id: i32,
    pub(crate) cliente: Option<TcpStream>,
    pub(crate) writer: Option<OwnedWriteHalf>,
}

impl Surtidor {
    pub fn new(
        id: usize,
        estacion: Addr<Estacion>,
        cliente: TcpStream,
        estacion_id: i32,
    ) -> Self {
        Surtidor {
            id,
            estacion,
            estacion_id,
            cliente: Some(cliente),
            writer: None,
        }
    }
}
impl Actor for Surtidor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("[{}] Surtidor conectado a la estación", self.estacion_id);

        // leer del stream del cliente la tarjeta y monto -> asumimos formato "tarjeta=monto\n"

        let estacion_addr = self.estacion.clone();
        let id_surtidor = self.id;
        let estacion_id = self.estacion_id;
        let cliente_actix = match self.cliente.take() {
            Some(s) => s,
            None => {
                println!("No hay cliente para leer");
                return;
            }
        };

        let std_stream = match cliente_actix.into_std() {
            Ok(s) => s,
            Err(e) => {
                println!("Error converting to std: {:?}", e);
                return;
            }
        };

        let tokio_stream = match tokio::net::TcpStream::from_std(std_stream) {
            Ok(s) => s,
            Err(e) => {
                println!("Error converting to tokio stream: {:?}", e);
                return;
            }
        };

        let (mut reader, writer) = tokio_stream.into_split();
        self.writer = Some(writer);

        actix_rt::spawn(async move {
            let mut buffer = [0u8; 128];
            match reader.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    let mensaje = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                    // parseo, chequeos...
                    let partes: Vec<&str> = mensaje.split('=').collect();
                    if partes.len() == 2 {
                        let monto: f32 = partes[1].parse().unwrap_or(0.0);
                        let venta = Venta {
                            id_venta: 0,
                            id_tarjeta: partes[0].parse().unwrap_or(0),
                            monto,
                            offline: false,
                            estado: EstadoVenta::Pendiente,
                            id_estacion: estacion_id,
                        };
                        estacion_addr.do_send(InformarVenta {
                            venta,
                            surtidor_id: id_surtidor,
                        });
                    } else {
                        println!("[{}] ({}) Formato inválido: {}", estacion_id, id_surtidor, mensaje);
                    }
                }
                Ok(_) => println!("[{}] ({}) Cliente desconectado", estacion_id, id_surtidor),
                Err(e) => println!("[{}] ({}) Error al leer: {:?}", estacion_id, id_surtidor, e),
            }
        });

    }
}