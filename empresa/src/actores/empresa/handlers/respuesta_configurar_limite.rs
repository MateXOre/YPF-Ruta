use actix::{Context, Handler};
use crate::actores::empresa::Empresa;
use crate::actores::empresa::messages::RespuestaConfigurarLimite;

impl Handler<RespuestaConfigurarLimite> for Empresa {
    type Result = ();

    fn handle(&mut self, msg: RespuestaConfigurarLimite, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.exito {
            println!(
                "[Empresa {}] ✓ Límite de tarjeta {} configurado exitosamente: {}",
                self.id, msg.id_tarjeta, msg.mensaje
            );
        } else {
            println!(
                "[Empresa {}] ✗ Error configurando límite de tarjeta {}: {}",
                self.id, msg.id_tarjeta, msg.mensaje
            );
        }
    }
}

