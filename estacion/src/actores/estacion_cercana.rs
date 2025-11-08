use actix::{Actor, Context, Handler, Message};

// ===== Mensajes =====
#[derive(Message)]
#[rtype(result = "()")]
pub struct ConectarEstacion(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct RespuestaConexion(pub String);

pub struct EstacionCercana {
    pub estacion_id: String,
    pub tx: tokio::sync::mpsc::Sender<String>, // para enviar líneas al writer task
}

impl Actor for EstacionCercana {
    type Context = Context<Self>;
}

impl Handler<ConectarEstacion> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: ConectarEstacion, _ctx: &mut Context<Self>) {
        let _ = self.tx.try_send(msg.0);
    }
}

impl Handler<RespuestaConexion> for EstacionCercana {
    type Result = ();

    fn handle(&mut self, msg: RespuestaConexion, _ctx: &mut Context<Self>) {
        println!("📨 [{}] recibió: {}", self.estacion_id, msg.0);
    }
}