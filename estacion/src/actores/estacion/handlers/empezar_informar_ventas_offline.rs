use actix::{Handler, Context, AsyncContext};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use std::collections::HashMap;
use tokio::time::sleep;
use std::time::Duration;
use actix::prelude::*;


impl Handler<EmpezarInformarVentasOffline> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: EmpezarInformarVentasOffline, ctx: &mut Context<Self>) {
        println!("[{}] Empezar a informar ventas offline solo si estoy online", self.id);
        if !self.estoy_conectada {
            println!("[{}] No estoy online, no informo ventas offline", self.id);
            return;
        }
        let ventas: HashMap<usize, HashMap<usize, Vec<util::structs::venta::Venta>>> = HashMap::new();
        let mensaje_bytes = InformarVentasOffline{id_lider: self.id, ventas}.to_bytes();

        self.enviar_a_siguiente(ctx, mensaje_bytes);

        if self.lider_actual == Some(self.id) {
            let addr = ctx.address();
            ctx.spawn(
                async move {
                    sleep(Duration::from_secs(Estacion::TIEMPO_INFORMAR_VENTAS_OFFLINE)).await;
                    addr.do_send(EmpezarInformarVentasOffline{});
                }
                .into_actor(self)
            );
        }
    }
}
