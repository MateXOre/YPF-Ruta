use crate::actores::gestor::messages::ModificarLimiteParticular;
use crate::actores::ypf::YpfRuta;
use crate::actores::ypf::messages::{ConfigurarLimite, EnviarBytesEmpresa};
use actix::{AsyncContext, Context, Handler};
use util::log_info;

impl Handler<ConfigurarLimite> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: ConfigurarLimite, ctx: &mut Context<Self>) -> Self::Result {
        log_info!(
            self.logger,
            "YpfRuta {}: Recibido ConfigurarLimite para tarjeta {} con monto {}",
            self.id,
            msg.id_tarjeta,
            msg.monto
        );

        let gestor_addr = self.gestor_addr.clone();
        let logger = self.logger.clone();
        let ypf_ruta_id = self.id;
        let ypf_ruta_addr = ctx.address();
        let empresa_id = msg.id_empresa;
        let id_tarjeta = msg.id_tarjeta;

        // Enviar al gestor para modificar el límite particular
        actix::spawn(async move {
            let (exito, mensaje) = match gestor_addr
                .send(ModificarLimiteParticular {
                    id_tarjeta,
                    nuevo_limite: msg.monto as u64,
                })
                .await
            {
                Ok(Ok(())) => {
                    log_info!(
                        logger,
                        "YpfRuta {}: Límite de tarjeta {} modificado exitosamente",
                        ypf_ruta_id,
                        id_tarjeta
                    );
                    (
                        true,
                        format!(
                            "Límite de tarjeta {} modificado exitosamente a {}",
                            id_tarjeta, msg.monto
                        ),
                    )
                }
                Ok(Err(e)) => {
                    eprintln!(
                        "YpfRuta {}: Error modificando límite de tarjeta {}: {}",
                        ypf_ruta_id, id_tarjeta, e
                    );
                    (false, format!("Error modificando límite: {}", e))
                }
                Err(e) => {
                    eprintln!(
                        "YpfRuta {}: Error enviando mensaje al gestor: {}",
                        ypf_ruta_id, e
                    );
                    (false, format!("Error interno: {}", e))
                }
            };

            // Enviar respuesta a la empresa
            let respuesta = serde_json::json!({
                "tipo": "ConfigurarLimite",
                "exito": exito,
                "mensaje": mensaje,
                "id_tarjeta": id_tarjeta,
                "id_empresa": empresa_id,
            });

            if let Ok(bytes) = serde_json::to_vec(&respuesta) {
                ypf_ruta_addr.do_send(EnviarBytesEmpresa { empresa_id, bytes });
            }
        });
    }
}
