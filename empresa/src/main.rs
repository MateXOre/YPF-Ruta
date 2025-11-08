use actix::prelude::*;
use tokio::{
    net::{TcpListener, TcpStream},
    io::{AsyncReadExt, AsyncWriteExt},
};
use std::net::SocketAddr;

// ======== MENSAJES ACTIX ========

// Mensaje Actix que representa algo recibido por TCP
#[derive(Message)]
#[rtype(result = "()")]
struct MensajeTCP {
    origen: SocketAddr,
    contenido: String,
}

// Mensaje Actix para que la estación envíe algo por TCP
#[derive(Message)]
#[rtype(result = "()")]
struct EnviarTCP {
    destino: SocketAddr,
    mensaje: String,
}

// ======== ACTOR PRINCIPAL ========

struct Estacion {
    id: u8,
}

impl Actor for Estacion {
    type Context = Context<Self>;
}

// Handler para mensajes recibidos por TCP
impl Handler<MensajeTCP> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: MensajeTCP, _: &mut Context<Self>) {
        if msg.contenido.trim().to_uppercase() == "SALUDO" {
            println!("[E{}] Recibí un saludo de {}", self.id, msg.origen);
        } else {
            println!(
                "[E{}] Recibió mensaje desde {:?}: {}",
                self.id, msg.origen, msg.contenido
            );
        }
    }
}

// Handler para enviar mensajes TCP
impl Handler<EnviarTCP> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: EnviarTCP, _: &mut Context<Self>) {
        let mensaje = msg.mensaje.clone();
        let destino = msg.destino;

        // Enviamos en una task de Tokio para no bloquear al actor
        tokio::spawn(async move {
            if let Ok(mut stream) = TcpStream::connect(destino).await {
                if let Err(e) = stream.write_all(mensaje.as_bytes()).await {
                    eprintln!("Error enviando mensaje a {:?}: {:?}", destino, e);
                } else {
                    println!("Mensaje enviado a {:?}: {}", destino, mensaje);
                }
            } else {
                eprintln!("No se pudo conectar con {:?}", destino);
            }
        });
    }
}

// ======== FUNCIONES AUXILIARES ========

impl Estacion {
    async fn escuchar_tcp(addr: Addr<Estacion>, puerto: u16) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", puerto))
            .await
            .expect("No se pudo bindear el puerto");
        println!("Estacion escuchando en puerto {}", puerto);

        loop {
            let (mut socket, peer) = listener.accept().await.unwrap();
            let addr_clone = addr.clone();

            tokio::spawn(async move {
                let mut buffer = vec![0u8; 1024];
                while let Ok(n) = socket.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    let msg = String::from_utf8_lossy(&buffer[..n]).to_string();
                    addr_clone.do_send(MensajeTCP {
                        origen: peer,
                        contenido: msg,
                    });
                }
            });
        }
    }
}

// ======== MAIN ========

#[actix::main]
async fn main() {
    // Creamos la estación 1 que escucha en 7001
    let estacion = Estacion { id: 1 }.start();
    tokio::spawn(Estacion::escuchar_tcp(estacion.clone(), 7001));

    // Ejemplo: luego de 2 segundos, enviamos un saludo a otro proceso (puerto 7002)
    let destino: SocketAddr = "127.0.0.1:7002".parse().unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    estacion.do_send(EnviarTCP {
        destino,
        mensaje: "SALUDO".into(),
    });

    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
}