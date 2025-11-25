use actix::{Context, Handler};
use crate::actores::empresa::Empresa;
use crate::actores::empresa::messages::RespuestaConfigurarLimiteGeneral;

impl Handler<RespuestaConfigurarLimiteGeneral> for Empresa {
    type Result = ();

    fn handle(&mut self, msg: RespuestaConfigurarLimiteGeneral, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.exito {
            println!(
                "[Empresa {}] ✓ Límite general configurado exitosamente: {}",
                self.id, msg.mensaje
            );
        } else {
            println!(
                "[Empresa {}] ✗ Error configurando límite general: {}",
                self.id, msg.mensaje
            );
        }
    }
}

