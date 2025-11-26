use crate::actores::estacion::{EnviarVentasAgrupadas, Estacion};
use crate::actores::ypf::messages::EnviarYpf;
use crate::actores::ypf::ypf_ruta::Ypf;
use actix::{Actor, AsyncContext, Context, Handler, WrapFuture};
use std::collections::HashMap;
use util::log_error;
use util::structs::venta::Venta;

fn serializar_ventas(
    ventas: &HashMap<usize, HashMap<usize, Vec<Venta>>>,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec(ventas).map_err(|e| format!("Error serializando ventas: {}", e))
}

fn generar_mensaje(
    ventas: &HashMap<usize, HashMap<usize, Vec<Venta>>>,
) -> Result<EnviarYpf, String> {
    let payload = serializar_ventas(ventas)?;
    let mut bytes = Vec::with_capacity(4 + payload.len());
    let len = payload.len() as u32;

    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(&payload);

    Ok(EnviarYpf { bytes })
}

impl Handler<EnviarVentasAgrupadas> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: EnviarVentasAgrupadas, ctx: &mut Context<Self>) {
        let self_addr = ctx.address();
        let ventas = self.ventas_por_informar.clone();

        let logger = self.logger.clone();
        ctx.spawn(
            async move {
                match Ypf::new(self_addr.clone(), logger.clone()).await {
                    Ok(ypf) => {
                        let addr = ypf.start();
                        match generar_mensaje(&ventas) {
                            Ok(mensaje) => {
                                let _ = addr.send(mensaje).await;
                            }
                            Err(e) => {
                                log_error!(
                                    logger,
                                    "Error generando mensaje de ventas agrupadas: {}",
                                    e
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log_error!(
                            logger,
                            "Error creando Ypf para enviar ventas agrupadas: {:?}",
                            e
                        );
                    }
                }
            }
            .into_actor(self),
        );

        self.ventas_por_informar.clear();
        self.limpiar_ventas_sin_informar();
        self.temporizador_activo = false;
    }
}
