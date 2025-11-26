use crate::actores::gestor::gestor_actor::Gestor;
use actix::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
mod actores;
use crate::actores::ypf::ypf_actor::YpfRuta;

#[actix::main]
async fn main() {
    println!("Iniciando YPF RUTA...");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <index> [<lider>]", args[0]);
        return;
    }

    let index: usize = args[1]
        .parse()
        .expect("El índice debe ser un número válido.");
    let file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("data")
        .join("ypfs.config");
    let log_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("logs")
        .join(format!("ypf_ruta_{}.log", index));

    let logger = util::logs::logger::Logger::new(log_path.to_str().unwrap()).unwrap_or_else(|| {
        eprintln!("Error al inicializar el logger en {:?}", log_path);
        std::process::exit(1);
    });

    let f = File::open(&file_path).unwrap_or_else(|e| {
        eprintln!(
            "Error al abrir archivo de configuración {:?}: {}",
            file_path, e
        );
        std::process::exit(1);
    });

    let mut peers = HashMap::new();
    let mut local = (0, 0);
    let lider = if args.len() >= 3 {
        if args[2].to_lowercase() == "lider" || args[2] == "1" {
            Some(index)
        } else {
            None
        }
    } else {
        None
    };

    for l in std::io::BufReader::new(f).lines().skip(1).flatten() {
        let parts: Vec<&str> = l.split(',').collect();
        if parts.len() != 2 {
            eprintln!("Línea de configuración inválida: {}", l);
            continue;
        }

        let id: usize = parts[0].parse().expect("ID inválido en configuración.");
        let puerto: usize = parts[1].parse().expect("Puerto inválido en configuración.");

        if id == index {
            local = (id, puerto);
        } else {
            peers.insert(
                id,
                std::net::SocketAddr::from(([127, 0, 0, 1], puerto as u16)),
            );
        }
    }

    let log_gestor = logger.get_log_channel();
    let log_ypf = log_gestor.clone();

    let gestor = Gestor::new(index, log_gestor).start();

    let _ypf_server = YpfRuta::new(local.0, local.1, lider, peers, gestor, log_ypf).start();

    println!("YPF RUTA {} iniciado en puerto {}", local.0, local.1);
    std::future::pending::<()>().await;
}
