use crate::actores::peer::messages::EleccionOk;
use crate::actores::ypf::ypf_actor::YpfRuta;
use actix::{Context, Handler};

impl Handler<EleccionOk> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: EleccionOk, _ctx: &mut Context<Self>) -> Self::Result {
        let responder_id = msg.0;
        println!(
            "YpfRuta {}: Recibido OK del nodo {}. Cancelo mi candidatura.",
            self.id, responder_id
        );
        self.respuestas_recibidas += 1;
        // No me declaro líder, espero que el nodo con mayor ID lo haga
    }
}
