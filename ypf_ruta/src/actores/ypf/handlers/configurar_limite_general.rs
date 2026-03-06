use crate::actores::gestor::messages::ModificarLimiteGeneral;
use crate::actores::ypf::YpfRuta;
use crate::actores::ypf::messages::{ConfigurarLimiteGeneral, EnviarBytesEmpresa};
use actix::{AsyncContext, Context, Handler};
use util::log_info;

impl Handler<ConfigurarLimiteGeneral> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: ConfigurarLimiteGeneral, ctx: &mut Context<Self>) -> Self::Result {
        log_info!(
            self.logger,
            "YpfRuta {}: Recibido ConfigurarLimiteGeneral para empresa {} con monto {}",
            self.id,
            msg.id_empresa,
            msg.monto
        );

        let gestor_addr = self.gestor_addr.clone();
        let logger = self.logger.clone();
        let ypf_ruta_id = self.id;
        let ypf_ruta_addr = ctx.address();
        let empresa_id = msg.id_empresa;

        // Enviar al gestor para modificar el límite general
        actix::spawn(async move {
            let (exito, mensaje) = match gestor_addr
                .send(ModificarLimiteGeneral {
                    id_empresa: msg.id_empresa,
                    nuevo_limite: msg.monto as u64,
                })
                .await
            {
                Ok(Ok(())) => {
                    log_info!(
                        logger,
                        "YpfRuta {}: Límite general de empresa {} modificado exitosamente",
                        ypf_ruta_id,
                        msg.id_empresa
                    );
                    (
                        true,
                        format!(
                            "Límite general de empresa {} modificado exitosamente a {}",
                            msg.id_empresa, msg.monto
                        ),
                    )
                }
                Ok(Err(e)) => {
                    eprintln!(
                        "YpfRuta {}: Error modificando límite general de empresa {}: {}",
                        ypf_ruta_id, msg.id_empresa, e
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
                "tipo": "ConfigurarLimiteGeneral",
                "exito": exito,
                "mensaje": mensaje,
                "id_empresa": empresa_id,
            });

            if let Ok(bytes) = serde_json::to_vec(&respuesta) {
                ypf_ruta_addr.do_send(EnviarBytesEmpresa { empresa_id, bytes });
            }
        });
    }
}
