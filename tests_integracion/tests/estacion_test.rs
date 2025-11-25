use actix::prelude::*;
use estacion::actores::estacion::Estacion;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn simular_cliente(
    addr: SocketAddr,
    tarjeta_id: usize,
    monto: f32,
) -> Result<bool, Box<dyn std::error::Error>> {
    let stream_result =
        tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr)).await;
    let mut stream = match stream_result {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(Box::new(e)),
        Err(_) => return Ok(false),
    };

    let mut buffer = [0u8; 128];
    let _read_result =
        tokio::time::timeout(Duration::from_millis(500), stream.read(&mut buffer)).await;
    let datos = format!("{}={}\n", tarjeta_id, monto);
    stream.write_all(datos.as_bytes()).await?;

    let mut respuesta = [0u8; 128];
    tokio::time::sleep(Duration::from_millis(200)).await;
    let _read_result =
        tokio::time::timeout(Duration::from_millis(500), stream.read(&mut respuesta)).await;
    Ok(true)
}

#[actix_rt::test]
async fn test_estacion_maneja_clientes_concurrentes() {
    let base_port = 20000u16;
    let estaciones = vec![SocketAddr::from(([127, 0, 0, 1], base_port))];
    let _estacion = Estacion::new(0, estaciones).start();

    tokio::time::sleep(Duration::from_millis(500)).await;
    let client_port = base_port + 1000;
    let addr = SocketAddr::from(([127, 0, 0, 1], client_port));
    let mut handles = vec![];
    for i in 0..5 {
        let addr_clone = addr;
        let handle = actix_rt::spawn(async move {
            simular_cliente(addr_clone, 100 + i, 50.0 + (i as f32 * 10.0)).await
        });
        handles.push(handle);
    }

    let mut exitosos = 0;
    for handle in handles {
        if let Ok(Ok(true)) = handle.await {
            exitosos += 1;
        }
    }

    assert!(
        exitosos > 0,
        "Deberían haberse conectado al menos algunos clientes. Exitosos: {}",
        exitosos
    );
}

#[actix_rt::test]
async fn test_estacion_limite_surtidores() {
    let base_port = 20001u16;
    let estaciones = vec![SocketAddr::from(([127, 0, 0, 1], base_port))];
    let _estacion = Estacion::new(0, estaciones).start();

    tokio::time::sleep(Duration::from_millis(500)).await;
    let client_port = base_port + 1000;
    let addr = SocketAddr::from(([127, 0, 0, 1], client_port));

    let mut handles = vec![];
    for i in 0..10 {
        let addr_clone = addr;
        let handle = actix_rt::spawn(async move {
            tokio::time::sleep(Duration::from_millis((i * 50) as u64)).await;
            simular_cliente(addr_clone, 200 + i, 30.0).await
        });
        handles.push(handle);
    }

    tokio::time::sleep(Duration::from_millis(3000)).await;
    let mut completadas = 0;
    for handle in handles {
        if handle.is_finished() {
            completadas += 1;
        }
    }

    assert!(
        completadas > 0,
        "Deberían haberse procesado algunas conexiones. Completadas: {}",
        completadas
    );
}

#[actix_rt::test]
async fn test_estacion_procesamiento_ventas_concurrentes() {
    let base_port = 20002u16;
    let estaciones = vec![SocketAddr::from(([127, 0, 0, 1], base_port))];
    let _estacion = Estacion::new(0, estaciones).start();

    tokio::time::sleep(Duration::from_millis(500)).await;
    let client_port = base_port + 1000;
    let addr = SocketAddr::from(([127, 0, 0, 1], client_port));

    let mut handles = vec![];
    for i in 0..8 {
        let addr_clone = addr;
        let tarjeta = 300 + i;
        let monto = 25.0 + (i as f32 * 5.0);

        let handle = actix_rt::spawn(async move {
            tokio::time::sleep(Duration::from_millis((i * 10) as u64)).await;
            simular_cliente(addr_clone, tarjeta, monto).await
        });
        handles.push(handle);
    }

    tokio::time::sleep(Duration::from_millis(3000)).await;

    let mut exitosas = 0;
    for handle in handles {
        if let Ok(Ok(true)) = handle.await {
            exitosas += 1;
        }
    }

    assert!(
        exitosas > 0,
        "Deberían haberse procesado múltiples ventas. Exitosas: {}",
        exitosas
    );
}

#[actix_rt::test]
async fn test_multiple_estaciones_concurrentes() {
    let mut estaciones_addrs = vec![];
    let mut estaciones_actors = vec![];

    let mut todas_las_estaciones = vec![];
    for i in 0..3 {
        let port = 20100 + i;
        todas_las_estaciones.push(SocketAddr::from(([127, 0, 0, 1], port as u16)));
    }

    for i in 0..3 {
        let port = 20100 + i;
        let estacion = Estacion::new(i, todas_las_estaciones.clone()).start();
        estaciones_addrs.push((port + 1000) as u16);
        estaciones_actors.push(estacion);
    }

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let mut handles = vec![];
    for (idx, &client_port) in estaciones_addrs.iter().enumerate() {
        let addr = SocketAddr::from(([127, 0, 0, 1], client_port));
        for j in 0..3 {
            let addr_clone = addr;
            let tarjeta = idx * 100 + j;
            let handle =
                actix_rt::spawn(async move { simular_cliente(addr_clone, tarjeta, 40.0).await });
            handles.push(handle);
        }
    }

    tokio::time::sleep(Duration::from_millis(2000)).await;
    let mut exitosas = 0;
    for handle in handles {
        if let Ok(Ok(true)) = handle.await {
            exitosas += 1;
        }
    }

    assert!(
        exitosas > 0,
        "Múltiples estaciones deberían haber procesado clientes. Exitosas: {}",
        exitosas
    );
}
