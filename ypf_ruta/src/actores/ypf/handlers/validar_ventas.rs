use actix::{Context, Handler};
use crate::actores::estacion::estacion_actor::ValidarVentas;
use crate::actores::ypf::ypf_actor::YpfRuta;

impl Handler<ValidarVentas> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: ValidarVentas, _ctx: &mut Context<Self>) -> Self::Result {
        println!("YpfRuta {}: Recibidas {} ventas para validar", self.id, msg.ventas.len());
        
        // Agregar a la cola de ventas por confirmar
        self.ventas_por_confirmar.push_back((msg.from, msg.ventas));
        
        println!("YpfRuta {}: Ventas agregadas a la cola. Total en cola: {}", self.id, self.ventas_por_confirmar.len());
    }
}
