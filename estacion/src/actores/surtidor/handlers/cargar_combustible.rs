use crate::actores::estacion::CobrarACliente;
use crate::actores::surtidor::{messages::CargarCombustible, surtidor::Surtidor};
use actix::{AsyncContext, Context, Handler, WrapFuture};

impl Handler<CargarCombustible> for Surtidor {
    type Result = ();

    fn handle(&mut self, msg: CargarCombustible, ctx: &mut Context<Self>) {
        let estacion = self.estacion.clone();
        let surtidor_id = self.id;
        let estacion_id = self.estacion_id;
        let venta = msg.venta;

        ctx.spawn(
            async move {
                println!(
                    "[{}] ({}) Cargando combustible...",
                    estacion_id, surtidor_id
                );

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                println!("[{}] ({}) Carga terminada", estacion_id, surtidor_id);

                estacion.do_send(CobrarACliente { venta, surtidor_id });
            }
            .into_actor(self),
        );
    }
}
