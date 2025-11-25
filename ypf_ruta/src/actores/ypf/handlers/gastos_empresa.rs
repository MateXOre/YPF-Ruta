use crate::actores::ypf::YpfRuta;
use crate::actores::ypf::messages::{GastosEmpresa, EnviarBytesEmpresa};
use crate::actores::gestor::messages::ConsultarEstado;
use actix::{AsyncContext, Context, Handler};
use util::log_info;

impl Handler<GastosEmpresa> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: GastosEmpresa, ctx: &mut Context<Self>) -> Self::Result {
        log_info!(
            self.logger,
            "YpfRuta {}: Recibido GastosEmpresa para empresa {}",
            self.id,
            msg.id_empresa
        );
        
        let gestor_addr = self.gestor_addr.clone();
        let logger = self.logger.clone();
        let ypf_ruta_id = self.id;
        let ypf_ruta_addr = ctx.address();
        let empresa_id = msg.id_empresa;
        
        // Enviar al gestor para consultar el estado
        actix::spawn(async move {
            let respuesta = match gestor_addr.send(ConsultarEstado(empresa_id)).await {
                Ok(Some((empresa, tarjetas))) => {
                    log_info!(
                        logger,
                        "YpfRuta {}: Estado de empresa {} consultado exitosamente",
                        ypf_ruta_id,
                        empresa_id
                    );

                    log_info!(
                        logger,
                        "YpfRuta {}: Estado de empresa {} consultado exitosamente: empresa:{:?}, tarjetas:{:?}",
                        ypf_ruta_id,
                        empresa_id,
                        empresa,
                        tarjetas
                    );
                    
                    serde_json::json!({
                        "tipo": "GastosEmpresa",
                        "exito": true,
                        "empresa": empresa,
                        "tarjetas": tarjetas
                    })
                }
                Ok(None) => {
                    eprintln!("YpfRuta {}: Empresa {} no encontrada", ypf_ruta_id, empresa_id);
                    serde_json::json!({
                        "tipo": "GastosEmpresa",
                        "exito": false,
                        "mensaje": format!("Empresa {} no encontrada", empresa_id)
                    })
                }
                Err(e) => {
                    eprintln!("YpfRuta {}: Error consultando estado de empresa {}: {}", ypf_ruta_id, empresa_id, e);
                    serde_json::json!({
                        "tipo": "GastosEmpresa",
                        "exito": false,
                        "mensaje": format!("Error interno: {}", e)
                    })
                }
            };
            
            // Enviar respuesta a la empresa
            if let Ok(bytes) = serde_json::to_vec(&respuesta) {
                ypf_ruta_addr.do_send(EnviarBytesEmpresa {
                    empresa_id,
                    bytes,
                });
            }
        });
    }
}

