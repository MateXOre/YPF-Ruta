use actix::{Handler, Context};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::*;
use actix::prelude::*;
use std::collections::HashMap;


impl Handler<EmpezarInformarVentasOffline> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: EmpezarInformarVentasOffline, ctx: &mut Context<Self>) {
        println!("[{}] Empezar a informar ventas offline", self.id);
        let ventas: HashMap<usize, HashMap<usize, Vec<util::structs::venta::Venta>>> = HashMap::new();
        let mensaje_bytes = InformarVentasOffline{id_lider: self.id, ventas}.to_bytes();

        self.enviar_a_siguiente(ctx, mensaje_bytes);
    }
}