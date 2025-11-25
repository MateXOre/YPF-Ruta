use actix::Actor;
use actix::{AsyncContext, Context, Handler};

use crate::actores::estacion::{AceptarCliente, Estacion, HabilitarSurtidor};
use crate::actores::surtidor::surtidor::Surtidor;
use actix::prelude::*;

impl Handler<AceptarCliente> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: AceptarCliente, ctx: &mut Context<Self>) {
        if self.surtidores.len() < self.max_surtidores {
            let id_surtidor = rand::random::<u64>() as usize;
            let estacion_addr = ctx.address();
            let estacion_id = self.id_global;

            println!(
                "[{}] habilitando surtidor {} para cliente {:?}",
                self.id, id_surtidor, msg.peer_addr
            );

            let stream = msg.stream;
            ctx.spawn(
                async move {
                    let surtidor =
                        Surtidor::new(id_surtidor, estacion_addr.clone(), stream, estacion_id);
                    let surtidor_addr = surtidor.start();

                    estacion_addr.do_send(HabilitarSurtidor {
                        surtidor_id: id_surtidor,
                        surtidor_addr,
                    });
                }
                .into_actor(self),
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
