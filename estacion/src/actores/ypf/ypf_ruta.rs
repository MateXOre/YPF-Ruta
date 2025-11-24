use std::collections::HashMap;
use actix::{Actor, Addr, Context};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use crate::actores::estacion::{Estacion, TransaccionesPorEstacion};

const YPF_ADDRS: [&str; 3] = ["127.0.0.1:18080", "127.0.0.1:18081", "127.0.0.1:18082"];

pub struct Ypf {
    writer: Option<OwnedWriteHalf>,
    reader: Option<OwnedReadHalf>,
    estacion_addr: Addr<Estacion>,
}

impl Ypf {
    pub async fn new(
        estacion_addr: Addr<Estacion>,
    ) -> Result<Self, ()> {
        for addr in YPF_ADDRS.iter() {
            println!("Intentando conectarnos a YPF en {}", addr);
            if let Ok(socket) = tokio::net::TcpStream::connect(addr).await {
                println!("Conectado a YPF en {}", addr);

                let (reader, writer) = socket.into_split();

                return Ok(Ypf {
                    writer: Some(writer),
                    reader: Some(reader),
                    estacion_addr,
                })
            } else {
                println!("No se pudo conectar a YPF en {}", addr);
            }
        }

        Err(())
    }

    pub fn enviar_por_socket(&mut self, bytes: Vec<u8>) {
        let mut writer_opt = if let Some(w) = self.writer.take() {
            w
        } else {
            eprintln!("No hay writer disponible para enviar datos a YPF.");
            return;
        };
        tokio::spawn(async move {
            if let Err(e) = writer_opt.write_all(&bytes).await {
                eprintln!("Error enviando datos a YPF: {}", e);
            }
        });
    }

    pub fn leer_de_socket(&mut self) {
        let mut reader = if let Some(r) = self.reader.take() {
            r
        } else {
            eprintln!("No hay reader disponible para leer datos de YPF.");
            return;
        };
        let estacion = self.estacion_addr.clone();
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match reader.read(&mut buf).await {
                    Ok(bytes) => {
                        
                        
                        
                        let transacciones: HashMap<usize, HashMap<usize, Vec<(usize, bool)>>> = HashMap::new(); // Reemplazar con el parseo real
                        estacion.do_send(TransaccionesPorEstacion{transacciones});
                        break; // Salir del loop después de una lectura para este ejemplo
                    }
                    Err(e) => {
                        eprintln!("Error leyendo de YPF: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

impl Actor for Ypf {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Actor Ypf iniciado.");
        self.leer_de_socket();
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Ypf detenido.");
    }
}




