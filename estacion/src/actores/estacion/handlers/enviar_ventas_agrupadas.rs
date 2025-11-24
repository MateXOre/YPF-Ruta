use std::collections::HashMap;
use actix::{Context, Handler, AsyncContext, Actor, WrapFuture};
use util::structs::venta::Venta;
use crate::actores::estacion::{EnviarVentasAgrupadas, Estacion};
use crate::actores::ypf::messages::EnviarYpf;
use crate::actores::ypf::ypf_ruta::Ypf;

fn serializar_ventas(ventas: &HashMap<usize, HashMap<usize, Vec<Venta>>>) -> Result<Vec<u8>, String> {
    serde_json::to_vec(ventas)
        .map_err(|e| format!("Error serializando ventas: {}", e))
}

fn generar_mensaje(
    ventas: &HashMap<usize, HashMap<usize, Vec<Venta>>>,
) -> Result<EnviarYpf, String> {
    let payload = serializar_ventas(ventas)?;
    let mut bytes = Vec::with_capacity(4 + payload.len());
    let len = payload.len() as u32;

    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(&payload);

    Ok(EnviarYpf{ bytes })
}

impl Handler<EnviarVentasAgrupadas> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: EnviarVentasAgrupadas, ctx: &mut Context<Self>) {
        let self_addr = ctx.address();
        let ventas = self.ventas_por_informar.clone(); // acá quizas conviene directamente crear el mensaje en vez de clonar pero bueno

        ctx.spawn(
            async move {
                match Ypf::new(self_addr.clone()).await {
                    Ok(ypf) => {
                        let addr = ypf.start();
                        match generar_mensaje(&ventas) {
                            Ok(mensaje) => {
                                let _ = addr.send(mensaje).await;
                            }
                            Err(e) => {
                                eprintln!("Error generando mensaje de ventas agrupadas: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creando Ypf para enviar ventas agrupadas: {:?}", e);
                    }
                }
            }
            .into_actor(self),
        );

        self.ventas_por_informar.clear(); // TODO: manejar caso de ypf caida en el medio
        self.temporizador_activo = false;
    }
}