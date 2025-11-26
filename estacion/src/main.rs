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

    let log_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("logs")
        .join(format!("ypf_ruta_{}.log", index_estacion));
    let log_ypf = util::logs::logger::Logger::new(log_path.to_str().unwrap()).unwrap_or_else(|| {
        eprintln!("Error al inicializar el logger en {:?}", log_path);
        std::process::exit(1);
    });

    let _estacion = Estacion::new(index_estacion, estaciones, log_ypf.get_log_channel()).start();

    tokio::signal::ctrl_c()
        .await
        .expect("Error esperando señal de terminación");

    println!("Se apaga la estacion {}", index_estacion);
}
