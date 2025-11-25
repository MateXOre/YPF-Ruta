use crate::actores::estacion::{Estacion, InformarVenta, NuevoLiderConectado};
use crate::actores::estacion_cercana::Enviar;
use actix::{AsyncContext, Handler};

impl Handler<NuevoLiderConectado> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: NuevoLiderConectado, ctx: &mut Self::Context) -> Self::Result {
        println!(
            "[{}] Nuevo líder conectado, Enviando ventas pendientes de confirmacion",
            self.id,
        );

        if self.lider_actual == Some(self.id) {
            println!(
                "[{}] Soy el líder actual, me envio a mi mismo las ventas pendientes de confirmacion",
                self.id,
            );

            if let Some(ventas_pendientes) = self.ventas_por_informar.get(&self.id) {
                println!(
                    "[{}] Tengo ventas pendientes de confirmacion: {:?}",
                    self.id, ventas_pendientes,
                );
                for (id_surtidor, ventas) in ventas_pendientes {
                    println!(
                        "[{}] Reenviando ventas del surtidor {}: {:?}",
                        self.id, id_surtidor, ventas
                    );
                    for venta in ventas {
                        println!("[{}] Reenviando venta: {:?}", self.id, venta);
                        let venta = venta.clone();

                        ctx.address().do_send(InformarVenta {
                            venta: venta.clone(),
                            id_surtidor: *id_surtidor,
                            id_estacion: self.id,
                        });

                        self.ventas_a_confirmar.insert(*id_surtidor, venta);
                    }
                }
            }
            self.ventas_por_informar.clear();
            self.limpiar_ventas_sin_informar();

            // como soy líder, no reenvío a ningún otro líder
            return;
        }

        // 🔥 Si NO soy el líder (reenvío al líder nuevo)
        if let Some(ventas_pendientes) = self.ventas_por_informar.get(&self.id) {
            if let Some(lider) = self.buscar_estacion_lider() {
                for (id_surtidor, ventas) in ventas_pendientes {
                    for venta in ventas {
                        let venta = venta.clone();
                        let mensaje = InformarVenta {
                            venta,
                            id_surtidor: *id_surtidor,
                            id_estacion: self.id,
                        };
                        lider.do_send(Enviar {
                            bytes: mensaje.to_bytes(),
                        });
                    }
                }
            }
        }
        self.ventas_por_informar.clear();
        self.limpiar_ventas_sin_informar();
    }
}
