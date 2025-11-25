use crate::actores::estacion::{Estacion, TransaccionesPorEstacion};
use actix::{Actor, Addr, Context};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

const YPF_ADDRS: [&str; 3] = ["127.0.0.1:18080", "127.0.0.1:18081", "127.0.0.1:18082"];

pub struct Ypf {
    writer: Option<OwnedWriteHalf>,
    reader: Option<OwnedReadHalf>,
    estacion_addr: Addr<Estacion>,
}

impl Ypf {
    pub async fn new(estacion_addr: Addr<Estacion>) -> Result<Self, ()> {
        for addr in YPF_ADDRS.iter() {
            println!("Intentando conectarnos a YPF en {}", addr);
            if let Ok(socket) = tokio::net::TcpStream::connect(addr).await {
                println!("Conectado a YPF en {}", addr);

                let (reader, writer) = socket.into_split();

                return Ok(Ypf {
                    writer: Some(writer),
                    reader: Some(reader),
                    estacion_addr,
                });
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
        let reader = if let Some(r) = self.reader.take() {
            r
        } else {
            eprintln!("No hay reader disponible para leer datos de YPF.");
            return;
        };
        let estacion = self.estacion_addr.clone();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let mut buf_reader = tokio::io::BufReader::new(reader);
            let mut line = Vec::new();

            match buf_reader.read_until(b'\n', &mut line).await {
                Ok(0) => {
                    eprintln!("Conexión cerrada por YPF.");
                }
                Ok(_) => {
                    if line.last() == Some(&b'\n') {
                        line.pop();
                    }

                    let transacciones: HashMap<usize, HashMap<usize, Vec<(usize, bool)>>> =
                        match serde_json::from_slice(&line) {
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!("Error deserializando datos de YPF: {}", e);
                                eprintln!("Datos recibidos: {:?}", String::from_utf8_lossy(&line));
                                return;
                            }
                        };
                    estacion.do_send(TransaccionesPorEstacion { transacciones });
                }
                Err(e) => {
                    eprintln!("Error leyendo de YPF: {}", e);
                }
            }
        });
    }
}

impl Actor for Ypf {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Ypf iniciado.");
        self.leer_de_socket();
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Actor Ypf detenido.");
    }
}
