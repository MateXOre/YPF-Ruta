use crate::actores::estacion::io::{handle_stream_incoming, handle_stream_outgoing};
use crate::actores::estacion::messages::*;
use crate::actores::estacion_cercana::Enviar;
use crate::actores::estacion_cercana::EstacionCercana;
use crate::actores::surtidor::surtidor::Surtidor;
use actix::prelude::*;
use actix::{Actor, Addr, Context};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

use crate::loader::estacion_loader::EstacionLoader;
use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use util::structs::venta::Venta;

pub struct Estacion {
    pub(crate) desconectada: bool,
    pub(crate) id: usize,
    pub(crate) port: u16,
    pub(crate) lider_actual: Option<usize>, // id de la estación líder actual
    pub(crate) siguiente_estacion: usize,   // id de la siguiente estación
    pub(crate) estaciones_cercanas: HashMap<usize, Addr<EstacionCercana>>, // id_estacion, addr actor estacion cercana
    pub(crate) total_estaciones: usize,
    pub(crate) todas_las_estaciones: HashMap<usize, SocketAddr>, // id_estacion, socketaddr
    pub(crate) primer_anillo_realizado: bool,
    pub(crate) ventas_a_confirmar: HashMap<usize, Venta>, // id_surtidor, venta (es solo una porque estas son ventas online)
    pub(crate) surtidores: HashMap<usize, Addr<Surtidor>>,
    pub(crate) max_surtidores: usize,
    pub(crate) cola_espera: VecDeque<AceptarCliente>,
    pub(crate) ventas_por_informar: HashMap<usize, HashMap<usize, Vec<Venta>>>, //id_estacion, id_surtidor, ventas es un vector porque cuando levantemos las offline puede haber más siempre podemos plantear no agruparlas en el mismo vector
    pub(crate) temporizador_activo: bool,
    pub(crate) listener_activo: Arc<AtomicBool>, // Controla si el listener debe seguir aceptando conexiones
    pub(crate) estoy_conectada: bool,
    pub(crate) id_global: usize,
}

impl Estacion {
    pub const DESPLAZAMIENTO_PUERTO_ESCUCHA_CLIENTES: u16 = 1000;
    pub const DESPLAZAMIENTO_PUERTO_ESCUCHA_CAMBIO_LISTENER: u16 = 3000;
    pub const TIEMPO_INFORMAR_VENTAS_OFFLINE: u64 = 30;
    pub const MAX_ESTACIONES_REGISTRADAS: usize = 5;

    pub fn new(index_estacion: usize, estaciones: Vec<SocketAddr>) -> Self {
        let mi_id_regional = index_estacion % Estacion::MAX_ESTACIONES_REGISTRADAS;
        let siguiente = if mi_id_regional + 1 < estaciones.len() {
            mi_id_regional + 1
        } else {
            0
        };

        Self {
            desconectada: false,
            id: mi_id_regional,
            port: estaciones[mi_id_regional].port(),
            lider_actual: None,
            estaciones_cercanas: HashMap::new(),
            siguiente_estacion: siguiente,
            total_estaciones: estaciones.len(),
            todas_las_estaciones: estaciones.into_iter().enumerate().collect(),
            primer_anillo_realizado: false,
            ventas_a_confirmar: HashMap::new(),
            surtidores: HashMap::new(),
            max_surtidores: 4,
            cola_espera: VecDeque::new(),
            ventas_por_informar: HashMap::new(),
            temporizador_activo: false,
            listener_activo: Arc::new(AtomicBool::new(true)),
            estoy_conectada: true,
            id_global: index_estacion,
        }
    }

    pub async fn connect_and_register(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
        id_destino: usize,
    ) {
        let _ = Estacion::intentar_conectar(target, actor_addr, id, id_destino).await;
    }

    pub(crate) async fn intentar_conectar(
        target: SocketAddr,
        actor_addr: Addr<Estacion>,
        id: usize,
        id_destino: usize,
    ) -> Result<(), std::io::Error> {
        match TcpStream::connect(target).await {
            Ok(stream) => {
                println!(
                    "[{}] conectado a {} con dirección:{}",
                    id, id_destino, target
                );
                let addr_clone = actor_addr.clone();
                if let Err(e) = handle_stream_outgoing(stream, id, addr_clone, id_destino).await {
                    eprintln!("[{}] error al manejar conexión con {}: {:?}", id, target, e);
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("[{}] no pudo conectar a {}: {}", id, target, e);
                Err(e)
            }
        }
    }

    pub(crate) fn enviar_a_siguiente(&self, ctx: &mut Context<Self>, mensaje: Vec<u8>) {
        if let Some(siguiente) = self.estaciones_cercanas.get(&self.siguiente_estacion) {
            siguiente.do_send(Enviar {
                bytes: mensaje.clone(),
            });
        } else {
            println!(
                "[{}] La siguiente estación {} no está conectada, no se puede mandar el mensaje de anillo",
                self.id, self.siguiente_estacion
            );
            ctx.address().do_send(EstacionDesconectada {
                estacion_id: self.siguiente_estacion,
                mensaje: mensaje.clone(),
            });
        };
    }

    pub(crate) fn buscar_estacion_lider(&self) -> Option<Addr<EstacionCercana>> {
        println!("Buscando lider actual: {:?}", self.lider_actual);
        if let Some(lider) = self.lider_actual {
            if let Some(conexion) = self.estaciones_cercanas.get(&lider) {
                return Some(conexion.clone());
            }
        }
        None
    }

    pub(crate) fn agregar_ventas_acumuladas(
        &mut self,
        mut ventas_acumuladas: HashMap<usize, HashMap<usize, Vec<Venta>>>,
    ) -> HashMap<usize, HashMap<usize, Vec<Venta>>> {
        let ventas_por_informar = self.ventas_por_informar.clone();
        for (id_estacion, surtidores_acumuladas) in ventas_por_informar {
            let entry_estacion_acumulada = ventas_acumuladas.entry(id_estacion).or_default();

            for (id_surtidor, mut ventas_acumuladas_surtidor) in surtidores_acumuladas {
                let entry_surtidor = entry_estacion_acumulada.entry(id_surtidor).or_default();

                entry_surtidor.append(&mut ventas_acumuladas_surtidor);
            }
        }
        ventas_acumuladas
    }

    pub(crate) fn cargar_ventas_sin_informar(&mut self) {
        let ventas_sin_informar = EstacionLoader::new(self.id_global).load_ventas_sin_informar();
        match ventas_sin_informar {
            Ok(ventas_sin_informar) => {
                self.ventas_por_informar = ventas_sin_informar;
            }
            Err(e) => {
                eprintln!("Error al cargar ventas sin informar: {:?}", e);
            }
        }
    }

    pub(crate) fn guardar_ventas_sin_informar(&mut self) {
        let ventas_sin_informar =
            EstacionLoader::new(self.id_global).save_ventas_sin_informar(&self.ventas_por_informar);
        match ventas_sin_informar {
            Ok(_) => {
                println!("Ventas sin informar guardadas correctamente");
            }
            Err(e) => {
                eprintln!("Error al guardar ventas sin informar: {:?}", e);
            }
        }
    }

    pub(crate) fn limpiar_ventas_sin_informar(&mut self) {
        let ventas_sin_informar = EstacionLoader::new(self.id_global).clear_ventas_sin_informar();
        match ventas_sin_informar {
            Ok(_) => {
                println!("Ventas sin informar limpiadas correctamente");
            }
            Err(e) => {
                eprintln!("Error al limpiar ventas sin informar: {:?}", e);
            }
        }
    }
}

impl Actor for Estacion {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let port = self.port;
        let id = self.id;
        let addr_self = ctx.address();
        let addr_self_clone = addr_self.clone();
        let addr_self_clone_2 = addr_self.clone();
        println!(
            "[{}] escuchando conexiones de clientes en 127.0.0.1:{}",
            id,
            port + Estacion::DESPLAZAMIENTO_PUERTO_ESCUCHA_CLIENTES
        );

        self.cargar_ventas_sin_informar();

        actix_rt::spawn(async move {
            if let Ok(listener) = TcpListener::bind((
                "127.0.0.1",
                port + Estacion::DESPLAZAMIENTO_PUERTO_ESCUCHA_CLIENTES,
            ))
            .await
            {
                loop {
                    match listener.accept().await {
                        Ok((stream, peer_addr)) => {
                            println!(
                                "[{}] conexión de cliente entrante desde {:?}",
                                id, peer_addr
                            );
                            addr_self_clone.do_send(AceptarCliente { stream, peer_addr });
                        }
                        Err(e) => {
                            eprintln!("Error al aceptar conexión de cliente: {:?}", e);
                        }
                    }
                }
            }
        });

        println!(
            "[{}] escuchando conexiones de estaciones en 127.0.0.1:{}",
            id, port
        );

        let listener_activo = Arc::clone(&self.listener_activo);
        actix_rt::spawn(async move {
            let listener: TcpListener = match TcpListener::bind(("127.0.0.1", port)).await {
                Ok(listener) => listener,
                Err(e) => {
                    eprintln!("Error al crear listener en puerto {}: {:?}", port, e);
                    return;
                }
            };

            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        let esta_activo = listener_activo.load(Ordering::Relaxed);
                        if !esta_activo {
                            println!(
                                "[{}] Listener detenido (activo={}), rechazando conexión de {:?}",
                                id, esta_activo, peer_addr
                            );
                            drop(stream); // Cerrar la conexión
                            continue;
                        }

                        println!(
                            "[{}] conexión de estación entrante desde {:?}",
                            id, peer_addr
                        );

                        let addr_clone = addr_self.clone();
                        actix_rt::spawn(async move {
                            if let Err(e) = handle_stream_incoming(stream, id, addr_clone).await {
                                eprintln!("Error manejando conexión entrante: {:?}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error al aceptar conexión: {:?}", e);
                    }
                }
            }
        });

        println!(
            "[{}] escuchando conexiones para cambiar conexión listener en 127.0.0.1:{}",
            id,
            port + Estacion::DESPLAZAMIENTO_PUERTO_ESCUCHA_CAMBIO_LISTENER
        );

        actix_rt::spawn(async move {
            if let Ok(listener) = TcpListener::bind((
                "127.0.0.1",
                port + Estacion::DESPLAZAMIENTO_PUERTO_ESCUCHA_CAMBIO_LISTENER,
            ))
            .await
            {
                loop {
                    match listener.accept().await {
                        Ok((stream, peer_addr)) => {
                            println!(
                                "[{}] conexión entrante para cambiar conexión listener desde {:?}",
                                id, peer_addr
                            );
                            addr_self_clone_2.do_send(CambiarConexionListener { stream });
                        }
                        Err(e) => {
                            eprintln!(
                                "Error al aceptar conexión para cambiar conexión listener: {:?}",
                                e
                            );
                        }
                    }
                }
            }
        });

        let addr_self_clone = ctx.address();
        if let Some(sig_addr) = self
            .todas_las_estaciones
            .get(&self.siguiente_estacion)
            .cloned()
        {
            println!(
                "Conectando con siguiente estación {}",
                self.siguiente_estacion
            );
            let siguiente_estacion = self.siguiente_estacion;
            actix_rt::spawn(async move {
                Estacion::connect_and_register(sig_addr, addr_self_clone, id, siguiente_estacion)
                    .await;
            });
        }
    }
}
