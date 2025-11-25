use crate::actores::gestor::messages::RegistrarVenta;
use crate::actores::peer::messages::VentaRegistrada;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Context, Handler};

impl Handler<VentaRegistrada> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: VentaRegistrada, _ctx: &mut Context<Self>) -> Self::Result {
        self.gestor_addr.do_send(RegistrarVenta(msg.venta));
    }
}
