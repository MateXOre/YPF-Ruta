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
            // Mostrar empresa
            if let Some(ref empresa) = msg.empresa {
                if let Ok(formatted) = serde_json::to_string_pretty(empresa) {
                    println!("Empresa: {}", formatted);
                }
            }
            // Mostrar tarjetas
            if let Some(ref tarjetas) = msg.tarjetas {
                if let Ok(formatted) = serde_json::to_string_pretty(tarjetas) {
                    println!("Tarjetas: {}", formatted);
                }
            }
        } else {
            let mensaje = msg.mensaje.unwrap_or_else(|| "Error desconocido".to_string());
            println!(
                "[Empresa {}] ✗ Error consultando gastos: {}",
                self.id, mensaje
            );
        }
    }
}

