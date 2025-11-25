use actix::{Context, Handler};
use crate::actores::empresa::Empresa;
use crate::actores::empresa::messages::RespuestaGastosEmpresa;

impl Handler<RespuestaGastosEmpresa> for Empresa {
    type Result = ();

    fn handle(&mut self, msg: RespuestaGastosEmpresa, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.exito {
            println!(
                "[Empresa {}] ✓ Gastos consultados exitosamente:",
                self.id
            );
            // Mostrar los datos de forma legible
            if let Ok(formatted) = serde_json::to_string_pretty(&msg.data) {
                println!("{}", formatted);
            } else {
                println!("{:?}", msg.data);
            }
        } else {
            println!(
                "[Empresa {}] ✗ Error consultando gastos: {}",
                self.id, msg.mensaje
            );
        }
    }
}

