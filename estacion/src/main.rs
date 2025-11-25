use actix::prelude::*;

mod actores;
mod errors;
mod loader;
use actores::estacion::*;
use loader::addr_loader::AddrLoader;

#[actix_rt::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let index_estacion: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let loader = AddrLoader::new();

    let rango_minimo = (index_estacion / Estacion::MAX_ESTACIONES_REGISTRADAS)
        * Estacion::MAX_ESTACIONES_REGISTRADAS;
    let rango_maximo = rango_minimo + Estacion::MAX_ESTACIONES_REGISTRADAS - 1;

    let estaciones = match loader.load_all(rango_minimo, rango_maximo) {
        Ok(estaciones) => estaciones,
        Err(e) => {
            return println!("{}", e);
        }
    };

    let _estacion = Estacion::new(index_estacion, estaciones).start();

    tokio::signal::ctrl_c()
        .await
        .expect("Error esperando señal de terminación");

    println!("Se apaga la estacion {}", index_estacion);
}
