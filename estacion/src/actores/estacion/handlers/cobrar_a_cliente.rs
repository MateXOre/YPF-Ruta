use crate::actores::estacion::messages::CobrarACliente;
use crate::actores::estacion::messages::InformarVenta;
use crate::actores::estacion::Estacion;
use crate::actores::estacion_cercana::Enviar;
use crate::actores::surtidor::messages::ResultadoVenta;
use actix::{AsyncContext, Context, Handler};
use util::log_info;
use util::log_warning;

impl Handler<CobrarACliente> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: CobrarACliente, ctx: &mut Context<Self>) {
        log_info!(
            self.logger,
            "[{}] Cobranza informada: {:?} por surtidor: {}",
            self.id,
            msg.venta.id_venta,
            msg.surtidor_id
        );
        if self.lider_actual.is_none() {
            self.estoy_conectada = false;
        }

        if self.lider_actual == Some(self.id) {
            ctx.address().do_send(InformarVenta {
                venta: msg.venta.clone(),
                id_surtidor: msg.surtidor_id,
                id_estacion: self.id,
            });

            self.ventas_a_confirmar.insert(msg.surtidor_id, msg.venta);
        } else {
            log_info!(self.logger, "[{}] validando venta con lider", self.id);
            if self.estoy_conectada {
                log_info!(
                    self.logger,
                    "[{}] Estoy conectada, enviando venta al lider",
                    self.id
                );
                if let Some(lider_addr) = self.buscar_estacion_lider() {
                    let venta_mensaje = InformarVenta {
                        venta: msg.venta.clone(),
                        id_surtidor: msg.surtidor_id,
                        id_estacion: self.id,
                    };
                    lider_addr.do_send(Enviar {
                        bytes: venta_mensaje.to_bytes(),
                    });
                    self.ventas_a_confirmar.insert(msg.surtidor_id, msg.venta);
                }
            } else {
                log_warning!(
                    self.logger,
                    "[{}] Estoy desconectada, me guardo la venta como offline",
                    self.id
                );
                let mut venta = msg.venta.clone();
                self.surtidores
                    .get(&msg.surtidor_id)
                    .unwrap()
                    .do_send(ResultadoVenta { exito: true });
                venta.offline = true;
                self.ventas_por_informar
                    .entry(self.id)
                    .or_default()
                    .entry(msg.surtidor_id)
                    .or_default()
                    .push(venta);
                self.guardar_ventas_sin_informar();
            }
        }
    }
}
