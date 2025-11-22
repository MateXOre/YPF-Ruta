use actix::{Context, Handler};
use crate::actores::ypf::messages::NuevoLider;
use crate::actores::ypf::ypf_actor::YpfRuta;

impl Handler<NuevoLider> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: NuevoLider, _ctx: &mut Context<Self>) -> Self::Result {
        println!("YpfRuta {}: Recibido anuncio de COORDINATOR del nodo {}", self.id, msg.id);
        
        // Aceptar el nuevo líder y cancelar cualquier elección en curso
        self.lider = Some(msg.id);
        self.en_eleccion = false;
        self.respuestas_recibidas = 0;
        
        println!("🎖️  YpfRuta {}: Nuevo líder confirmado: {}", self.id, msg.id);
    }
}