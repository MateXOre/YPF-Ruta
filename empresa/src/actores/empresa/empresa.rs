use actix::prelude::*;
use tokio::io::{self, AsyncBufReadExt, BufReader};

// Mensaje para procesar entrada de consola
#[derive(Message)]
#[rtype(result = "()")]
pub struct ResponderConsola {
    pub linea: String,
}

// Mensaje para procesar datos recibidos del socket
#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcesarMensajeSocket {
    pub bytes: Vec<u8>,
}

impl Handler<ResponderConsola> for Empresa {
    type Result = ();
    fn handle(&mut self, msg: ResponderConsola, _: &mut Context<Self>) {
        println!("[Empresa {}] Entrada recibida: {}", self.id, msg.linea);
    }
}

impl Handler<ProcesarMensajeSocket> for Empresa {
    type Result = ();
    fn handle(&mut self, msg: ProcesarMensajeSocket, _: &mut Context<Self>) {
        println!("[Empresa {}] Mensaje recibido del socket: {} bytes", self.id, msg.bytes.len());
        // Aquí puedes procesar los bytes según tus necesidades
    }
}

pub struct Empresa {
    id: usize,
}

impl Empresa {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

impl Actor for Empresa {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let empresa_addr = ctx.address();

        // Leer de la consola en una tarea asíncrona
        actix_rt::spawn(async move {
            let stdin = BufReader::new(io::stdin());
            let mut lines = stdin.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                // Enviar mensaje al actor
                empresa_addr.do_send(ResponderConsola { linea: line });
            }
        });





    }
}