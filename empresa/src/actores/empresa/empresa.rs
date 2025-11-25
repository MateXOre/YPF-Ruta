use actix::prelude::*;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use crate::actores::ypf_ruta::ypf_ruta::YpfRuta;
use crate::actores::empresa::messages::{ConectadoAypfRuta, ResponderConsola};

const YPF_ADDRS: [&str; 3] = ["127.0.0.1:18080", "127.0.0.1:18081", "127.0.0.1:18082"];

pub struct Empresa {
    pub id: usize,
    pub ypf_ruta_addr: Option<Addr<YpfRuta>>,
}

impl Empresa {
    pub fn new(id: usize) -> Self {
        Self { id, ypf_ruta_addr: None }
    }
}

impl Actor for Empresa {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let empresa_addr = ctx.address();

        //
        // 1. Conectarse a YPF Ruta
        //
        actix_rt::spawn({
            let empresa_addr = empresa_addr.clone();

            async move {
                for addr in YPF_ADDRS.iter() {
                    println!("Intentando conectarse a YpfRuta en {}", addr);
                    match tokio::net::TcpStream::connect(addr).await {
                        Ok(stream) => {
                            println!("Empresa conectada a YpfRuta en {}", addr);

                            let (reader, writer) = stream.into_split();

                            // crear actor YpfRuta
                            let ypf_ruta = YpfRuta::new(empresa_addr.clone(), reader, writer).start();

                            // avisar a la empresa que ya tenemos conexión
                            empresa_addr.do_send(ConectadoAypfRuta { addr: ypf_ruta });
                            break;
                        }
                        Err(e) => {
                            eprintln!("No se pudo conectar a {}: {}", addr, e);
                        }
                    }
                }
            }
        });


    }
}