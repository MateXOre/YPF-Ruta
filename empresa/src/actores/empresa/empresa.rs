use crate::actores::empresa::io::outgoing::handle_stream_outgoing;
use crate::actores::ypf_ruta::ypf_ruta::YpfRuta;
use actix::prelude::*;

const YPF_ADDRS: [&str; 3] = ["127.0.0.1:19080", "127.0.0.1:19081", "127.0.0.1:19082"];

pub struct Empresa {
    pub id: usize,
    pub ypf_ruta_addr: Option<Addr<YpfRuta>>,
}

impl Empresa {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            ypf_ruta_addr: None,
        }
    }
}

impl Actor for Empresa {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let empresa_addr = ctx.address();

        actix_rt::spawn({
            let empresa_addr = empresa_addr.clone();
            let id = self.id;
            async move {
                for addr in YPF_ADDRS.iter() {
                    println!("Intentando conectarse a YpfRuta en {}", addr);
                    match tokio::net::TcpStream::connect(addr).await {
                        Ok(stream) => {
                            println!("Empresa conectada a YpfRuta en {}", addr);

                            let _ = handle_stream_outgoing(stream, id, empresa_addr.clone()).await;
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
