use actix::prelude::*;

mod actores;

use actores::empresa::Empresa;

#[actix::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let index_empresa: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

    let _empresa = Empresa::new(index_empresa).start();
    tokio::signal::ctrl_c()
        .await
        .expect("Error esperando señal de terminación");

    println!("Se apaga la empresa {}", index_empresa);
}
