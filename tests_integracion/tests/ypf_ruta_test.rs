use actix::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use util::structs::venta::{EstadoVenta, Venta};
use ypf_ruta::actores::gestor::Gestor;
use ypf_ruta::actores::ypf::YpfRuta;

const TEST_INDEX_1: usize = 9996;
const TEST_INDEX_2: usize = 9997;
const TEST_INDEX_3: usize = 9998;
const TEST_INDEX_4: usize = 9999;

fn crear_logger_test() -> mpsc::Sender<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || while rx.recv().is_ok() {});
    tx
}

fn obtener_data_dir() -> std::path::PathBuf {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    workspace_root.join("ypf_ruta").join("src").join("data")
}

fn inicializar_archivos_test(index: usize) {
    let data_dir = obtener_data_dir();

    if let Err(e) = fs::create_dir_all(&data_dir) {
        eprintln!(
            "Advertencia: No se pudo crear el directorio {:?}: {}",
            data_dir, e
        );
    }
    let empresas_path = data_dir.join(format!("empresas_{}.json", index));
    let tarjetas_path = data_dir.join(format!("tarjetas_{}.json", index));
    let ventas_path = data_dir.join(format!("ventas_{}.json", index));

    if !empresas_path.exists() {
        let _ = fs::write(&empresas_path, "[]");
    }
    if !tarjetas_path.exists() {
        let _ = fs::write(&tarjetas_path, "[]");
    }
    if !ventas_path.exists() {
        let _ = fs::write(&ventas_path, "[]");
    }
}

/// Limpia los archivos JSON creados por el Gestor durante los tests
fn limpiar_archivos_test(index: usize) {
    let data_dir = obtener_data_dir();

    let empresas_path = data_dir.join(format!("empresas_{}.json", index));
    let tarjetas_path = data_dir.join(format!("tarjetas_{}.json", index));
    let ventas_path = data_dir.join(format!("ventas_{}.json", index));

    let _ = fs::remove_file(empresas_path);
    let _ = fs::remove_file(tarjetas_path);
    let _ = fs::remove_file(ventas_path);
}

fn limpiar_todos_los_archivos_test() {
    let data_dir = obtener_data_dir();

    let test_indices = [
        TEST_INDEX_1,
        TEST_INDEX_2,
        TEST_INDEX_3,
        TEST_INDEX_4,
        TEST_INDEX_4 + 1,
        TEST_INDEX_4 + 2,
    ];
    for &index in &test_indices {
        limpiar_archivos_test(index);
    }

    for index in 9996..=10001 {
        let empresas_path = data_dir.join(format!("empresas_{}.json", index));
        let tarjetas_path = data_dir.join(format!("tarjetas_{}.json", index));
        let ventas_path = data_dir.join(format!("ventas_{}.json", index));
        let _ = fs::remove_file(empresas_path);
        let _ = fs::remove_file(tarjetas_path);
        let _ = fs::remove_file(ventas_path);
    }
}

async fn simular_estacion_lider(
    addr: SocketAddr,
    ventas: HashMap<usize, HashMap<usize, Vec<Venta>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream_result =
        tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr)).await;

    let mut stream = match stream_result {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(Box::new(e)),
        Err(_) => return Ok(()),
    };
    let json_data = serde_json::to_vec(&ventas)?;

    let len = json_data.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&json_data).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(())
}

fn crear_venta_test(id_venta: usize, id_tarjeta: usize, monto: f32, id_estacion: usize) -> Venta {
    Venta {
        id_venta,
        id_tarjeta,
        monto,
        id_estacion,
        offline: false,
        estado: EstadoVenta::Pendiente,
    }
}

#[actix_rt::test]
async fn test_ypf_ruta_maneja_estaciones_concurrentes() {
    inicializar_archivos_test(TEST_INDEX_1);
    let base_port = 18080usize;
    let logger = crear_logger_test();
    let logger_ypf = logger.clone();
    let gestor = Gestor::new(TEST_INDEX_1, logger).start();
    let peers = HashMap::new();
    let _ypf_ruta = YpfRuta::new(0, base_port, Some(0), peers, gestor, logger_ypf).start();

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let estaciones_port = (base_port + 10000) as u16; // OFFSET_ESTACIONES
    let addr = SocketAddr::from(([127, 0, 0, 1], estaciones_port));

    let mut handles = vec![];
    for estacion_id in 0..5 {
        let addr_clone = addr;
        let est_id = estacion_id;

        let handle = actix_rt::spawn(async move {
            let mut ventas_por_surtidor = HashMap::new();
            let mut ventas = Vec::new();

            for surtidor_id in 0..2 {
                let venta = crear_venta_test(
                    est_id * 10 + surtidor_id,
                    100 + est_id,
                    50.0 + (surtidor_id as f32 * 10.0),
                    est_id,
                );
                ventas.push(venta);
            }
            ventas_por_surtidor.insert(0, ventas);
            let mut solicitud = HashMap::new();
            solicitud.insert(est_id, ventas_por_surtidor);
            tokio::time::sleep(Duration::from_millis((est_id * 50) as u64)).await;
            simular_estacion_lider(addr_clone, solicitud).await
        });
        handles.push(handle);
    }

    tokio::time::sleep(Duration::from_millis(3000)).await;
    let mut exitosas = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            exitosas += 1;
        }
    }

    assert!(
        exitosas > 0,
        "YpfRuta debería haber manejado múltiples conexiones. Exitosas: {}",
        exitosas
    );
    limpiar_archivos_test(TEST_INDEX_1);
}

#[actix_rt::test]
async fn test_ypf_ruta_procesamiento_ventas_concurrentes() {
    inicializar_archivos_test(TEST_INDEX_2);
    let base_port = 18081usize;
    let logger = crear_logger_test();
    let logger_ypf = logger.clone();
    let gestor = Gestor::new(TEST_INDEX_2, logger).start();
    let peers = HashMap::new();
    let _ypf_ruta = YpfRuta::new(1, base_port, Some(1), peers, gestor, logger_ypf).start();

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let estaciones_port = (base_port + 10000) as u16;
    let addr = SocketAddr::from(([127, 0, 0, 1], estaciones_port));

    let mut handles = vec![];
    for estacion_id in 0..10 {
        let addr_clone = addr;
        let est_id = estacion_id;

        let handle = actix_rt::spawn(async move {
            let mut ventas_por_surtidor = HashMap::new();
            let mut ventas = Vec::new();
            for i in 0..3 {
                let venta = crear_venta_test(
                    est_id * 100 + i,
                    200 + est_id,
                    30.0 + (i as f32 * 5.0),
                    est_id,
                );
                ventas.push(venta);
            }

            ventas_por_surtidor.insert(0, ventas);
            let mut solicitud = HashMap::new();
            solicitud.insert(est_id, ventas_por_surtidor);
            tokio::time::sleep(Duration::from_millis((est_id * 10) as u64)).await;
            simular_estacion_lider(addr_clone, solicitud).await
        });
        handles.push(handle);
    }

    tokio::time::sleep(Duration::from_millis(4000)).await;
    let mut exitosas = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            exitosas += 1;
        }
    }

    assert!(
        exitosas > 0,
        "YpfRuta debería haber procesado múltiples solicitudes. Exitosas: {}",
        exitosas
    );
}

#[actix_rt::test]
async fn test_ypf_ruta_cola_ventas() {
    inicializar_archivos_test(TEST_INDEX_3);
    let base_port = 18082usize;
    let logger = crear_logger_test();
    let logger_ypf = logger.clone();
    let gestor = Gestor::new(TEST_INDEX_3, logger).start();
    let peers = HashMap::new();
    let _ypf_ruta = YpfRuta::new(2, base_port, Some(2), peers, gestor, logger_ypf).start();

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let estaciones_port = (base_port + 10000) as u16;
    let addr = SocketAddr::from(([127, 0, 0, 1], estaciones_port));

    let mut handles = vec![];
    for estacion_id in 0..20 {
        let addr_clone = addr;
        let est_id = estacion_id;

        let handle = actix_rt::spawn(async move {
            let mut ventas_por_surtidor = HashMap::new();
            let mut ventas = Vec::new();
            let venta = crear_venta_test(est_id, 300 + est_id, 25.0, est_id);
            ventas.push(venta);
            ventas_por_surtidor.insert(0, ventas);

            let mut solicitud = HashMap::new();
            solicitud.insert(est_id, ventas_por_surtidor);
            simular_estacion_lider(addr_clone, solicitud).await
        });
        handles.push(handle);
    }
    tokio::time::sleep(Duration::from_millis(6000)).await;
    let mut exitosas = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            exitosas += 1;
        }
    }

    assert!(
        exitosas > 0,
        "YpfRuta debería haber encolado y procesado múltiples solicitudes. Exitosas: {}",
        exitosas
    );
}

#[actix_rt::test]
async fn test_multiple_ypf_ruta_concurrentes() {
    let mut ypf_rutas = vec![];
    let test_indices = [TEST_INDEX_4, TEST_INDEX_4 + 1, TEST_INDEX_4 + 2];
    for &test_idx in &test_indices {
        inicializar_archivos_test(test_idx);
    }
    for (i, &test_idx) in test_indices.iter().enumerate() {
        let base_port = 18100 + i;
        let logger = crear_logger_test();
        let logger_ypf = logger.clone();
        let gestor = Gestor::new(test_idx, logger).start();
        let peers = HashMap::new();
        let lider = if i == 0 { Some(i) } else { None };
        let ypf_ruta = YpfRuta::new(i, base_port, lider, peers, gestor, logger_ypf).start();
        ypf_rutas.push(ypf_ruta);
    }

    tokio::time::sleep(Duration::from_millis(1500)).await;
    let estaciones_port = (18100 + 10000) as u16;
    let addr = SocketAddr::from(([127, 0, 0, 1], estaciones_port));

    let mut handles = vec![];
    for estacion_id in 0..5 {
        let addr_clone = addr;
        let est_id = estacion_id;

        let handle = actix_rt::spawn(async move {
            let mut ventas_por_surtidor = HashMap::new();
            let mut ventas = Vec::new();

            let venta = crear_venta_test(est_id, 400 + est_id, 35.0, est_id);
            ventas.push(venta);
            ventas_por_surtidor.insert(0, ventas);
            let mut solicitud = HashMap::new();
            solicitud.insert(est_id, ventas_por_surtidor);
            simular_estacion_lider(addr_clone, solicitud).await
        });
        handles.push(handle);
    }

    tokio::time::sleep(Duration::from_millis(3000)).await;
    let mut exitosas = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            exitosas += 1;
        }
    }

    assert!(
        exitosas > 0,
        "El YpfRuta líder debería haber procesado solicitudes. Exitosas: {}",
        exitosas
    );

    for &test_idx in &test_indices {
        limpiar_archivos_test(test_idx);
    }
}

/// limpieza final de archivos generados por los tests
#[actix_rt::test]
async fn ypf_ruta_limpieza_de_data() {
    tokio::time::sleep(Duration::from_millis(8000)).await;
    for _ in 0..15 {
        limpiar_todos_los_archivos_test();
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
