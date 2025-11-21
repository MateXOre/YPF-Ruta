use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use actix::prelude::*;
use crate::actores::gestor::gestor_actor::Gestor;
mod actores;
use crate::actores::ypf::ypf_actor::YpfRuta;

#[actix::main]
async fn main() {
    println!("Hello, world!");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <index>", args[0]);
        return;
    }

    println!("Loading file {}", args[1]);
    let index: usize = args[1].parse().expect("El índice debe ser un número válido.");
    let file_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("data")
        .join("ypfs.config");
    
    let f = File::open(&file_path).unwrap_or_else(|e| {
        eprintln!("Error al abrir archivo de configuración {:?}: {}", file_path, e);
        std::process::exit(1);
    });
    
    let mut peers = HashMap::new();
    let mut local = (0, 0, None);

    for line in std::io::BufReader::new(f).lines().skip(1) {
        if let Ok(l) = line {
            let parts: Vec<&str> = l.split(',').collect();
            if parts.len() != 3 {
                eprintln!("Línea de configuración inválida: {}", l);
                continue;
            }

            let id: usize = parts[0].parse().expect("ID inválido en configuración.");
            let puerto: usize = parts[1].parse().expect("Puerto inválido en configuración.");
            let lider: Option<usize> = if parts[2] == "true" {
                Some(id)
            } else {
                None
            };

            if id == index {
                local = (id, puerto, lider);
            } else {
                peers.insert(
                    id,
                    std::net::SocketAddr::from(([127, 0, 0, 1], puerto as u16)),
                );
            }
        }
    }

    let gestor = Gestor::new().start();

    let _ypf_server = YpfRuta::new(
        local.0,
        local.1,
        local.2,
        peers,
        gestor
    ).start();

    // no bloquees Actix con sleep, mejor mantené el sistema vivo:
    // System::current().run().await;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
