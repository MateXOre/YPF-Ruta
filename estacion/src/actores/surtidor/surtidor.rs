use std::io::Read;

use actix::{Actor, Addr, AsyncContext, Context, WrapFuture};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use util::structs::venta::{EstadoVenta, Venta};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::CobrarACliente;
use crate::actores::surtidor::messages::{CargarCombustible, Detenerme};
use tokio::sync::mpsc::UnboundedSender;

pub struct Surtidor {
    pub(crate) id: usize,
    pub(crate) estacion: Addr<Estacion>,
    pub(crate) estacion_id: usize,
    pub(crate) reader: Option<OwnedReadHalf>,
    pub(crate) writer_tx: UnboundedSender<Vec<u8>>,
}

impl Surtidor {
    pub fn new(id: usize, estacion: Addr<Estacion>, cliente: TcpStream, estacion_id: usize) -> Self {
        let (reader, writer) = cliente.into_split();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

        // Task que posee el writer y serializa las escrituras
        tokio::spawn(async move {
            let mut writer = writer;
            while let Some(buf) = rx.recv().await {
                if buf.is_empty() { // señal de cierre
                    break;
                }
                if let Err(e) = writer.write_all(&buf).await {
                    eprintln!("Error al escribir: {:?}", e);
                    break;
                }
            }
        });

        Surtidor {
            id,
            estacion,
            estacion_id,
            reader: Some(reader),
            writer_tx: tx,
        }
    }
    // Ejemplo de envío
    pub fn enviar_al_cliente(&self, data: &[u8]) {
        let _ = self.writer_tx.send(data.to_vec());
    }
}

impl Actor for Surtidor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("[{}] Surtidor conectado a la estación", self.estacion_id);


        // Enviar mensaje inicial al cliente
        let _ = self.writer_tx.send(b"Ingrese tarjeta=monto\n".to_vec());

        // leer del stream del cliente la tarjeta y monto -> asumimos formato "tarjeta=monto\n"
        let mut reader = match self.reader.take() {
            Some(r) => r,
            None => {
                println!(
                    "[{}] ({}) Error: reader no disponible al iniciar el surtidor",
                    self.estacion_id, self.id
                );
                return;
            }
        };
        let id_surtidor = self.id;
        let estacion_id = self.estacion_id;
        let surtidor_addr = ctx.address();
        let writer_tx = self.writer_tx.clone();

        actix_rt::spawn(async move {
            let mut buffer = [0u8; 128];
            let mut continuar = true;

            while continuar {
                let result = reader.read(&mut buffer).await;

                match result {
                    Ok(n) if n > 0 => {
                        let mensaje = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                        let partes: Vec<&str> = mensaje.split('=').collect();

                        if partes.len() == 2 {
                            let monto: f32 = match partes[1].parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    println!("Monto inválido: {}", partes[1]);
                                    continue;
                                }
                            };
                            let id: usize = match partes[0].parse() {
                                Ok(v) => v,
                                Err(_) => {
                                    println!("Tarjeta inválida: {}", partes[0]);
                                    continue;
                                }
                            };

                            let venta = Venta {
                                id_venta: 0,
                                id_tarjeta: id,
                                monto,
                                offline: false,
                                estado: EstadoVenta::Pendiente,
                                id_estacion: estacion_id,
                            };

                            surtidor_addr.do_send(CargarCombustible {
                                venta,
                            });
                            continuar = false;
                        } else {
                            let _ = writer_tx.send(b"Formato invalido, use tarjeta=monto\n".to_vec());
                            println!("[{}] ({}) Formato inválido: {}", estacion_id, id_surtidor, mensaje);
                        }
                    }
                    Ok(_) => {
                        println!("[{}] ({}) Cliente desconectado", estacion_id, id_surtidor);
                        surtidor_addr.do_send(Detenerme);
                        continuar = false;
                    }
                    Err(e) => {
                        println!("[{}] ({}) Error al leer: {:?}", estacion_id, id_surtidor, e);
                        surtidor_addr.do_send(Detenerme);
                        continuar = false;
                    }
                }
            }
        });


        /*Versión anterior la dejo por las dudas
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
                        estacion_addr.do_send(CobrarACliente {
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
        });*/

    }
}