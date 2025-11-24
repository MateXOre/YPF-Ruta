use actix::{Context, Handler};
use crate::actores::ypf::messages::NuevoLider;
use crate::actores::ypf::ypf_actor::YpfRuta;

impl Handler<NuevoLider> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: NuevoLider, ctx: &mut Context<Self>) -> Self::Result {
        println!("YpfRuta {}: Recibido anuncio de COORDINATOR del nodo {}", self.id, msg.id);
        
        // Aceptar el nuevo líder y cancelar cualquier elección en curso
        if self.id == msg.id {
            println!("🎖️  YpfRuta {}: He sido elegido líder.", self.id);
        } else {
            println!("🏅 YpfRuta {}: Nodo {} es el nuevo líder.", self.id, msg.id);   
        }
        self.lider = Some(msg.id);
        self.en_eleccion = false;
        self.respuestas_recibidas = 0;
        
        println!("🎖️  YpfRuta {}: Nuevo líder confirmado: {}", self.id, msg.id);

        if self.lider == Some(self.id) {
            self.escuchar_estaciones(ctx)
        }
    }
}