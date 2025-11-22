use std::fmt::Error;
use actix::prelude::*;
use crate::actores::gestor::structs::Venta;
use crate::actores::ypf::ypf_actor::YpfRuta;
use serde_json;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader};
pub(crate) use crate::actores::estacion::messages::ValidarVentas;

fn parse_msg_from_estacion(buffer: Vec<u8>) -> Result<Vec<Venta>, &'static str>{
    match std::str::from_utf8(&buffer) {
        Ok(s) => {
            match serde_json::from_str::<Vec<Venta>>(s.trim()) {
                Ok(vs) => {
                    println!("Estacion: parseadas {} ventas iniciales.", vs.len());
                    Ok(vs)
                }
                Err(e) => {
                    Err("Estacion: error parseando JSON de ventas iniciales")
                }
            }
        }
        Err(e) => {
            Err("Estacion: buffer no es UTF-8")
        }
    }
}


pub struct Estacion {
    pub(crate) socket: Option<TcpStream>,
    ypf_addr: Addr<YpfRuta>,
    ventas_iniciales: Option<Vec<Venta>>,
}

impl Estacion {
    pub async fn new(socket: TcpStream, ypf_addr: Addr<YpfRuta>) -> Result<Estacion, ()> {
        let mut reader = BufReader::new(socket);
        let mut buf: Vec<u8> = Vec::new();

        // Intentamos leer hasta '\n'. Si falla, buf queda vacío.
        match tokio::io::AsyncBufReadExt::read_until(&mut reader, b'\n', &mut buf).await {
            Ok(_n) => {
                let ventas = match parse_msg_from_estacion(buf) {
                    Ok(ventas) => ventas,
                    Err(e) => {
                        println!("{}", e);
                        return Err(());
                    }
                };

                let socket = reader.into_inner();
                Ok(Estacion {
                    socket: Some(socket),
                    ypf_addr,
                    ventas_iniciales: Some(ventas),
                })
            }
            Err(e) => {
                eprintln!("Estacion: error leyendo del socket inicial: {}", e);
                Err(())
            }
        }
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
