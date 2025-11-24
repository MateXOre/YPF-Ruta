use actix::{Context, Handler};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::CambiarConexionListener;
use crate::actores::estacion_cercana::CerrarConexion;
use std::sync::atomic::Ordering;

impl Handler<CambiarConexionListener> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: CambiarConexionListener, _ctx: &mut Context<Self>) {
        println!("[{}] Cambiando conexión - deteniendo listener y cerrando todas las conexiones", self.id);
        
        if self.listener_activo.load(Ordering::Relaxed) {
            self.listener_activo.store(false, Ordering::Relaxed);
            self.estoy_conectada = false;
            self.lider_actual = None;
            println!("[{}] Listener detenido", self.id);
        
            let estaciones_ids: Vec<usize> = self.estaciones_cercanas.keys().cloned().collect();
            
            for estacion_id in estaciones_ids {
                if let Some(estacion_addr) = self.estaciones_cercanas.remove(&estacion_id) {
                    println!("[{}] Cerrando conexión con estación {}", self.id, estacion_id);

                    estacion_addr.do_send(CerrarConexion);
                }
            }
            
            println!("[{}] Todas las conexiones cerradas. Total de estaciones cercanas: {}", 
                        self.id, self.estaciones_cercanas.len());
            
            drop(msg.stream);
            println!("[{}] Stream de nueva conexión cerrado", self.id);
        } else {
            println!("[{}] Vuelvo a empezar a escuchar conexiones", self.id);
            self.listener_activo.store(true, Ordering::Relaxed);
        }
    }
}

