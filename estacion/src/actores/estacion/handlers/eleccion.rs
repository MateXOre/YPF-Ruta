use actix::{Handler, Context};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use actix::prelude::*;


impl Handler<Eleccion> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Eleccion, ctx: &mut Context<Self>) {
        println!("[{}] recibió mensaje del anillo: {:?}", self.id, msg.aspirantes_ids);
        self.primer_anillo_realizado = true;

        if let Some(lider) = self.lider_actual {
            if lider == self.id {
                println!("[{}] Soy el líder, entonces no es necesario que siga el anillo", self.id);
                return;
            } 
        }



        if msg.aspirantes_ids.contains(&self.id) {
            if let Some(nuevo_lider) = msg.aspirantes_ids.iter().max().copied() {
                println!(
                    "[{}] Detecté que mi id está en la lista, el nuevo líder es {}.",
                    self.id, nuevo_lider
                );

                // Enviamos el mensaje solo si encontramos líder
                ctx.address().do_send(NotificarLider { id_lider: nuevo_lider, id_iniciador: self.id });
            }
            return;
        }

        let mut nuevos_aspirantes = msg.aspirantes_ids.clone();
        nuevos_aspirantes.push(self.id);
        println!("[{}] agregue mi id. Nuevos aspirantes: {:?}", self.id, nuevos_aspirantes);
        
        let mensaje_bytes = Eleccion{aspirantes_ids: nuevos_aspirantes.clone()}.to_bytes();
        self.enviar_a_siguiente(ctx, mensaje_bytes);
    }
}