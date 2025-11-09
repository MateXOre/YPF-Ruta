use actix::prelude::*;

mod actores;
use actores::estacion::*;
use std::collections::VecDeque;
use std::net::SocketAddr;

#[actix_rt::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let index_estacion: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    let estaciones: Vec<SocketAddr> = vec![
        "127.0.0.1:9000".parse().unwrap(),
        "127.0.0.1:9001".parse().unwrap(),
        "127.0.0.1:9002".parse().unwrap(),
    ];

    let estacion = Estacion::new(index_estacion, estaciones).start();

    tokio::signal::ctrl_c()
        .await
        .expect("Error esperando señal de terminación");
    
    println!("Se apaga la estacion {}", index_estacion);
}
