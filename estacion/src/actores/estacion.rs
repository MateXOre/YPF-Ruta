use actix::{Actor, Context, Handler, Message};
use actix::prelude::*;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::io::Write;

enum _VentaEstado {
    Pendiente,
    Aprobada,
    Rechazada,
}

struct _Venta {
    id_venta: i32,
    id_tarjeta: i32,
    id_estacion: i32,
    monto: f32,
    timestamp: i64,
    estado: _VentaEstado,
    offline: bool,
}

/// Mensaje de elección de líder a través del anillo
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Eleccion {
    pub aspirantes_ids: Vec<usize>,
    pub iniciador_id: usize, // ID de la estación que inició la elección
}

/// Mensaje de elección de líder a través del anillo
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct IniciarEstacion {
}

/// Mensaje de coordinación que notifica el nuevo líder
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Coordinador {
    pub id_lider: usize,
}

// ============================================================================
// Actor Estacion
// ============================================================================
pub struct Cliente {
    pub _id_cliente: i32,
}

pub struct Estacion {
    pub id_estacion: usize,
    pub estaciones_regionales: Vec<TcpStream>,
    pub id_lider: Option<usize>,
    pub clientes_en_cola: VecDeque<Cliente>, // Cola de clientes esperando
}

impl Actor for Estacion {
    type Context = Context<Self>;
}

// ============================================================================
// Handlers para Mensajes Recibidos
// ============================================================================

impl Handler<IniciarEstacion> for Estacion{
    type Result = ();
    
    fn handle(&mut self, _msg: IniciarEstacion, ctx: &mut Self::Context){
        println!("Me inicio con el mensaje IniciarEstacion siendo la Estacion {}", self.id_estacion);
        let nuevos_aspirantes = vec![self.id_estacion];
        ctx.address().do_send(Eleccion{
            aspirantes_ids: nuevos_aspirantes,
            iniciador_id: self.id_estacion as usize});
    }
}


/// Handler para Eleccion (mensaje del anillo)
impl Handler<Eleccion> for Estacion {
    type Result = ();

    fn handle(&mut self, _msg: Eleccion, _ctx: &mut Self::Context) {
        println!("Estación {} recibió Eleccion", self.id_estacion);

        // if msg.iniciador_id == self.id_estacion {
        //     // El mensaje volvió al iniciador, determinar nuevo líder
        //     let nuevo_lider_id = *msg.aspirantes_ids.iter().max().unwrap();
        //     println!("Estación {} determinó nuevo líder: {}", self.id_estacion, nuevo_lider_id);

        //     // Enviar mensaje de coordinación al ganador
        //     if let Some(ganador_addr) = self.estaciones_addrs.get(&nuevo_lider_id) {
        //         ganador_addr.do_send(Coordinador {
        //             id_lider: nuevo_lider_id,
        //         });
        //     }

        //     self.en_eleccion = false;
        // } else {
        //     // Agregar mi ID y reenviar al siguiente nodo
        //     let mut aspirantes_ids = msg.aspirantes_ids;
        //     aspirantes_ids.push(self.id_estacion);

        //     if let Some(siguiente_nodo) = self.obtener_siguiente_nodo() {
        //         if let Some(siguiente_addr) = self.estaciones_addrs.get(&siguiente_nodo) {
        //             siguiente_addr.do_send(Eleccion {
        //                 aspirantes_ids,
        //                 iniciador_id: msg.iniciador_id,
        //             });
        //         }
        //     }
        // }
    }
}

/// Handler para Coordinador (notificación de nuevo líder)
impl Handler<Coordinador> for Estacion {
    type Result = ();

    fn handle(&mut self, msg: Coordinador, _ctx: &mut Self::Context) {
        println!(
            "Estación {} recibió Coordinador, nuevo líder: {}",
            self.id_estacion, msg.id_lider
        );

        // let era_lider = self.es_lider();
        // self.id_lider = Some(msg.id_lider);

        // // Si ahora soy el líder, configurar timers
        // if self.es_lider() && !era_lider {
        //     self.configurar_timer_ventas(ctx);
        //     self.configurar_timer_ventas_offline(ctx);
        // }

        // // Reenviar el mensaje al siguiente nodo del anillo
        // if let Some(siguiente_nodo) = self.obtener_siguiente_nodo() {
        //     if let Some(siguiente_addr) = self.estaciones_addrs.get(&siguiente_nodo) {
        //         siguiente_addr.do_send(Coordinador {
        //             id_lider: msg.id_lider,
        //         });
        //     }
        // }
    }
}


impl Estacion {
    pub async fn new(id_estacion: usize, estaciones: Vec<SocketAddr>) -> Addr<Estacion> {
        let sockets = estaciones.into_iter().filter_map(|addr| TcpStream::connect(addr).ok()).collect();
        let estacion_dir = Estacion::create(
            
        )
        let estacion = Self {
            id_estacion,
            estaciones_regionales: sockets,
            id_lider: None,
            clientes_en_cola: VecDeque::new()
            };
        estacion.start()
    }

    // pub fn send_message(&self, id: usize, msg: impl Message + Send + 'static) {
    //     if estacion_id == self.id_estacion {
    //         self.
    //         return; // No enviarse a sí mismo
    //     }
    //     let socket_to_send = &self.estaciones_regionales[id];

    //     socket_to_send.write(msg);
    // }
}