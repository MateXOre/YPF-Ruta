use actix::{AsyncContext, Context, Handler};
use actix::{Actor};

use actix::prelude::*;
use crate::actores::estacion::{Estacion, AceptarCliente, HabilitarSurtidor};
use crate::actores::surtidor::surtidor::Surtidor;

impl Handler<AceptarCliente> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: AceptarCliente, ctx: &mut Context<Self>) {
        if self.surtidores.len() < self.max_surtidores {

            let id_surtidor = rand::random::<u64>() as usize;
            let estacion_addr = ctx.address();
            let estacion_id = self.id;

            println!(
                "[{}] habilitando surtidor {} para cliente {:?}",
                self.id, id_surtidor, msg.peer_addr
            );

            // movemos stream dentro del future
            let stream = msg.stream;

            ctx.spawn(
                async move {
                    let surtidor = Surtidor::new(id_surtidor, estacion_addr.clone(), stream, estacion_id);
                    let surtidor_addr = surtidor.start();

                    // avisamos al actor que el surtidor quedó listo
                    estacion_addr.do_send(HabilitarSurtidor {
                        surtidor_id: id_surtidor,
                        surtidor_addr,
                    });
                }
                    .into_actor(self) // IMPORTANTÍSIMO
            );
        } else {
            println!(
                "[{}] No hay surtidores disponibles, cliente {:?} queda en espera",
                self.id, msg.peer_addr
            );

            self.cola_espera.push_back(msg);
        }
    }
}