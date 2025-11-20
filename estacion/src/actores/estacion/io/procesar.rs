use actix::Addr;

use crate::actores::estacion::Estacion;
use crate::actores::estacion::messages::{Eleccion, NotificarLider};
use crate::actores::estacion_cercana::{EstacionCercana, RespuestaConexion};

pub fn procesar_mensaje(
    line: &str,
    server_addr: &Addr<Estacion>,
    conn: &Addr<EstacionCercana>,
) {
    if line.starts_with("ANILLO:") {
        let contenido = line.strip_prefix("ANILLO:").unwrap_or(line).trim();

        let aspirantes_ids = contenido
            .split(',')
            .filter_map(|s| s.trim().parse::<usize>().ok())
            .collect::<Vec<usize>>();

        server_addr.do_send(Eleccion { aspirantes_ids });

    } else if line.starts_with("LIDER:") {
        let contenido = line.strip_prefix("LIDER:").unwrap_or(line).trim();
        let partes: Vec<&str> = contenido.split(',').collect();

        if partes.len() == 2 {
            if let (Ok(id_lider), Ok(id_iniciador)) =
                (partes[0].parse::<usize>(), partes[1].parse::<usize>())
            {
                server_addr.do_send(NotificarLider { id_lider, id_iniciador });
            }
        }

    } 
    // else {
    //     // Mensaje normal → reenviar al actor de la conexión
    //     conn.do_send(RespuestaConexion(line.to_string()));
    // }
}