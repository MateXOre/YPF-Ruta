use actix::{Handler, Context};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use actix::prelude::*; 
use actix::ActorFutureExt;
use tokio::time::sleep;
use std::time::Duration;

impl Handler<NotificarLider> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: NotificarLider, ctx: &mut Context<Self>) {
        if self.id == msg.id_iniciador && self.lider_actual == Some(msg.id_lider){
            println!(
                "[{}] 🔁 Mensaje de líder {} completó el ciclo, fin de propagación.",
                self.id, msg.id_lider
            );
            let addr = ctx.address();
            ctx.spawn(
                async move {
                    sleep(Duration::from_secs(Estacion::TIEMPO_INFORMAR_VENTAS_OFFLINE)).await;
                    addr.do_send(EmpezarInformarVentasOffline{});
                }
                .into_actor(self)
            );
            return;
        }



        self.lider_actual = Some(msg.id_lider);

        println!(
            "[{}] Mi nuevo lider es : {} ",
            self.id, msg.id_lider
        );

        if self.id != msg.id_lider {

            // ✔️ PRIMERO verificamos si ya existe conexión
            if self.estaciones_cercanas.contains_key(&msg.id_lider) {
                println!(
                    "[{}] Ya tengo conexión activa con el líder {}, no abro un nuevo socket.",
                    self.id, msg.id_lider
                );
            } else if let Some(lider_addr) = self.todas_las_estaciones.get(&msg.id_lider).copied() {

                // ❗ Solo si NO existe conexión → intento conectar
                println!(
                    "[{}] intentando conectarme al nuevo líder en {}...",
                    self.id, lider_addr
                );

                let addr_self = ctx.address();
                let self_id = self.id;
                let nuevo_lider = msg.id_lider;

                ctx.spawn(
                    actix::fut::wrap_future(async move {
                        match Estacion::intentar_conectar(
                            lider_addr,
                            addr_self.clone(),
                            self_id,
                            nuevo_lider
                        ).await {
                            Ok(_) => println!("[{}] ✅ conexión establecida con el líder {}", self_id, nuevo_lider),
                            Err(_) => println!("[{}] ❌ no se pudo conectar con el líder {}", self_id, nuevo_lider),
                        }
                    })
                        .map(|_, _, _| ())
                );
            }

        } else {
            println!(
                "[{}] Soy el nuevo líder, no necesito conectarme a mí mismo.",
                self.id
            );
        }

        // Reenviamos el mensaje al siguiente en el anillo
        let mensaje_serializado = NotificarLider {id_lider: msg.id_lider.clone(), id_iniciador: msg.id_iniciador}.to_bytes();
        self.enviar_a_siguiente(ctx, mensaje_serializado);
    }
}