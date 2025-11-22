use actix::prelude::*;
use crate::actores::gestor::structs::Venta;
use crate::actores::ypf::ypf_actor::YpfRuta;
use serde_json;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader};

#[derive(Message)]
#[rtype(result = "()")]
pub struct ValidarVentas {
    pub ventas: Vec<Venta>,
    pub from: Addr<Estacion>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ResultadoVentas {
    pub ventas: Vec<Venta>,
}

pub struct Estacion {
    // Socket para escribir respuesta final (se consume al enviar ResultadoVentas).
    socket: Option<TcpStream>,
    // Direccion del actor YpfRuta al que enviar ValidarVentas.
    ypf_addr: Addr<YpfRuta>,
    // Ventas leidas inicialmente desde el socket (si las hubo).
    ventas_iniciales: Option<Vec<Venta>>,
}

impl Estacion {
    /// Crea una instancia de `Estacion` leyendo el mensaje inicial desde `socket`
    /// hasta '\n' (o hasta encontrar '\0' en el buffer) y parseándolo a JSON -> Vec<Venta>.
    /// Devuelve la instancia (lista para `.start()` por quien llame).
    pub async fn new(socket: TcpStream, ypf_addr: Addr<YpfRuta>) -> Self {
        // Usamos BufReader para leer hasta newline.
        let mut reader = BufReader::new(socket);
        let mut buf: Vec<u8> = Vec::new();

        // Intentamos leer hasta '\n'. Si falla, buf queda vacío.
        match tokio::io::AsyncBufReadExt::read_until(&mut reader, b'\n', &mut buf).await {
            Ok(_n) => {
                // Intentamos interpretar como UTF-8 y parsear JSON a Vec<Venta>.
                let ventas = match std::str::from_utf8(&buf) {
                    Ok(s) => {
                        match serde_json::from_str::<Vec<Venta>>(s.trim()) {
                            Ok(vs) => {
                                println!("Estacion: parseadas {} ventas iniciales.", vs.len());
                                vs
                            }
                            Err(e) => {
                                eprintln!("Estacion: error parseando JSON de ventas iniciales: {}. Buffer: {:?}", e, String::from_utf8_lossy(&buf));
                                Vec::new()
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Estacion: buffer no es UTF-8: {}", e);
                        Vec::new()
                    }
                };

                // Recuperamos el socket interno del reader para usarlo más tarde.
                let socket = reader.into_inner();
                Estacion {
                    socket: Some(socket),
                    ypf_addr,
                    ventas_iniciales: Some(ventas),
                }
            }
            Err(e) => {
                eprintln!("Estacion: error leyendo del socket inicial: {}", e);
                // Recuperamos el socket aunque la lectura falló.
                let socket = reader.into_inner();
                Estacion {
                    socket: Some(socket),
                    ypf_addr,
                    ventas_iniciales: None,
                }
            }
        }
    }
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Estacion actor iniciado.");
        // Si teníamos ventas iniciales leídas, las enviamos a YpfRuta para validar.
        if let Some(ventas) = self.ventas_iniciales.take() {
            let from = ctx.address();
            // Construimos el mensaje y lo enviamos sin esperar respuesta (do_send).
            // YpfRuta deberá implementar el handler correspondiente para recibir ValidarVentas.
            self.ypf_addr.do_send(ValidarVentas { ventas, from });
            println!("Estacion: envié ValidarVentas a YpfRuta.");
        } else {
            println!("Estacion: no se recibieron ventas iniciales al conectar.");
        }
    }
}

impl Handler<ResultadoVentas> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: ResultadoVentas, ctx: &mut Context<Self>) -> Self::Result {
        println!("Estacion: recibí ResultadoVentas con {} ventas.", msg.ventas.len());

        // Si tenemos socket, escribimos la respuesta y cerramos.
        if let Some(socket) = self.socket.take() {
            // Movemos el socket al async block y lo ejecutamos como futura actix.
            let fut = async move {
                let mut s = socket;
                // Serializamos a JSON.
                match serde_json::to_vec(&msg.ventas) {
                    Ok(mut bytes) => {
                        // Añadimos newline como terminador (convention).
                        bytes.push(b'\n');
                        if let Err(e) = s.write_all(&bytes).await {
                            eprintln!("Estacion: error escribiendo ResultadoVentas al socket: {}", e);
                        } else {
                            println!("Estacion: ResultadoVentas escrito en socket correctamente.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Estacion: error serializando ResultadoVentas: {}", e);
                    }
                }
                // Intentamos cerrar la conexión de forma ordenada.
                let _ = s.shutdown().await;
            };

            // Convertimos la futura en un actor-future para poder detener el actor al terminar.
            ctx.spawn(
                fut.into_actor(self)
                    .map(|_, _act, ctx| {
                        // Al terminar la escritura, detenemos el actor (termina la conexión)
                        ctx.stop();
                    }),
            );
        } else {
            // No hay socket: simplemente detenemos el actor.
            println!("Estacion: no hay socket para enviar ResultadoVentas, finalizando actor.");
            ctx.stop();
        }
    }
}