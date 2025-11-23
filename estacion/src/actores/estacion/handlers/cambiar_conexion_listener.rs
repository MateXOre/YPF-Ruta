use actix::{Context, Handler};
use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::CambiarConexionListener;
use crate::actores::estacion_cercana::CerrarConexion;
use std::sync::atomic::Ordering;

impl Handler<CambiarConexionListener> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: CambiarConexionListener, _ctx: &mut Context<Self>) {
        println!("[{}] Cambiando conexión - deteniendo listener y cerrando todas las conexiones", self.id);
        
        // 1. Detener el listener estableciendo listener_activo a false
        self.listener_activo.store(false, Ordering::Relaxed);
        println!("[{}] Listener detenido", self.id);
        
        // 2. Dropear todas las conexiones en estaciones_cercanas
        // Para cerrar las conexiones, necesitamos detener los actores EstacionCercana
        // Al detener los actores, se cerrarán los sockets automáticamente
        let estaciones_ids: Vec<usize> = self.estaciones_cercanas.keys().cloned().collect();
        
        for estacion_id in estaciones_ids {
            if let Some(estacion_addr) = self.estaciones_cercanas.remove(&estacion_id) {
                println!("[{}] Cerrando conexión con estación {}", self.id, estacion_id);
                // Enviar mensaje para cerrar limpiamente la conexión y detener el actor
                // Esto cerrará el sender, terminando el task de escritura, y luego detendrá el actor
                estacion_addr.do_send(CerrarConexion);
            }
        }
        
        println!("[{}] Todas las conexiones cerradas. Total de estaciones cercanas: {}", 
                 self.id, self.estaciones_cercanas.len());
        
        // 3. Dropear el stream de la nueva conexión que llegó
        drop(msg.stream);
        println!("[{}] Stream de nueva conexión cerrado", self.id);
    }
}

