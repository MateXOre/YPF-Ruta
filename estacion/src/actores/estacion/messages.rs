use crate::actores::{estacion_cercana::EstacionCercana, surtidor::surtidor::Surtidor};
use actix::{Addr, Message};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use util::structs::venta::Venta;

// ===== Opcodes del protocolo =====
const OPCODE_REENVIAR: u8 = 0x01;
const OPCODE_ELECCION: u8 = 0x02;
const OPCODE_NOTIFICAR_LIDER: u8 = 0x03;
const OPCODE_INFORMAR_VENTA: u8 = 0x04;
const OPCODE_CONFIRMAR_TRANSACCIONES: u8 = 0x05;
pub const OPCODE_IDENTIFICAR: u8 = 0x06;
pub const OPCODE_INFORMAR_VENTAS_OFFLINE: u8 = 0x08;
const OPCODE_TRANSACCIONES_POR_ESTACION: u8 = 0x09;

// ===== Funciones helper para serialización binaria =====
fn write_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn read_u64(buf: &[u8], offset: &mut usize) -> Result<u64, String> {
    if *offset + 8 > buf.len() {
        return Err("Buffer insuficiente para leer u64".to_string());
    }
    let bytes = &buf[*offset..*offset + 8];
    *offset += 8;
    Ok(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

fn write_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn read_u32(buf: &[u8], offset: &mut usize) -> Result<u32, String> {
    if *offset + 4 > buf.len() {
        return Err("Buffer insuficiente para leer u32".to_string());
    }
    let bytes = &buf[*offset..*offset + 4];
    *offset += 4;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn write_f32(buf: &mut Vec<u8>, value: f32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn read_f32(buf: &[u8], offset: &mut usize) -> Result<f32, String> {
    if *offset + 4 > buf.len() {
        return Err("Buffer insuficiente para leer f32".to_string());
    }
    let bytes = &buf[*offset..*offset + 4];
    *offset += 4;
    Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn write_usize(buf: &mut Vec<u8>, value: usize) {
    write_u64(buf, value as u64);
}

fn read_usize(buf: &[u8], offset: &mut usize) -> Result<usize, String> {
    read_u64(buf, offset).map(|v| v as usize)
}

fn write_bool(buf: &mut Vec<u8>, value: bool) {
    buf.push(if value { 1 } else { 0 });
}

fn read_bool(buf: &[u8], offset: &mut usize) -> Result<bool, String> {
    if *offset >= buf.len() {
        return Err("Buffer insuficiente para leer bool".to_string());
    }
    let value = buf[*offset];
    *offset += 1;
    Ok(value != 0)
}

fn write_venta(buf: &mut Vec<u8>, venta: &Venta) {
    write_usize(buf, venta.id_venta);
    write_usize(buf, venta.id_tarjeta);
    write_usize(buf, venta.id_estacion);
    write_f32(buf, venta.monto);
    write_bool(buf, venta.offline);
    let estado_byte = match venta.estado {
        util::structs::venta::EstadoVenta::Pendiente => 0u8,
        util::structs::venta::EstadoVenta::Confirmada => 1u8,
        util::structs::venta::EstadoVenta::Rechazada => 2u8,
    };
    buf.push(estado_byte);
}

fn read_venta(buf: &[u8], offset: &mut usize) -> Result<Venta, String> {
    let id_venta = read_usize(buf, offset)?;
    let id_tarjeta = read_usize(buf, offset)?;
    let id_estacion = read_usize(buf, offset)?;
    let monto = read_f32(buf, offset)?;
    let offline = read_bool(buf, offset)?;
    if *offset >= buf.len() {
        return Err("Buffer insuficiente para leer estado".to_string());
    }
    let estado_byte = buf[*offset];
    *offset += 1;
    let estado = match estado_byte {
        0 => util::structs::venta::EstadoVenta::Pendiente,
        1 => util::structs::venta::EstadoVenta::Confirmada,
        2 => util::structs::venta::EstadoVenta::Rechazada,
        _ => util::structs::venta::EstadoVenta::Pendiente,
    };
    Ok(Venta {
        id_venta,
        id_tarjeta,
        id_estacion,
        monto,
        offline,
        estado,
    })
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct IdentificarEstacion {
    pub id: usize,
}

impl IdentificarEstacion {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![OPCODE_IDENTIFICAR];
        write_usize(&mut buf, self.id);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes[0] != OPCODE_IDENTIFICAR {
            return Err("Opcode incorrecto para identificar".into());
        }
        let mut offset = 1;
        let id = read_usize(bytes, &mut offset)?;
        Ok(Self { id })
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AgregarEstacion {
    pub estacion: Addr<EstacionCercana>,
    pub estacion_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SurtidorLibre {
    pub surtidor_id: usize,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Reenviar {
    pub bytes: Vec<u8>,
}

impl Reenviar {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }

        Ok(Reenviar {
            bytes: bytes.to_vec(),
        })
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Eleccion {
    pub aspirantes_ids: Vec<usize>,
}

impl Eleccion {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_ELECCION {
            return Err("Opcode incorrecto para Eleccion".to_string());
        }
        let mut offset = 1;
        let count = read_u32(bytes, &mut offset)? as usize;
        let mut aspirantes_ids = Vec::with_capacity(count);
        for _ in 0..count {
            aspirantes_ids.push(read_usize(bytes, &mut offset)?);
        }
        Ok(Eleccion { aspirantes_ids })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_ELECCION);
        write_u32(&mut buf, self.aspirantes_ids.len() as u32);
        for &id in &self.aspirantes_ids {
            write_usize(&mut buf, id);
        }
        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct NotificarLider {
    pub id_lider: usize,
    pub id_iniciador: usize,
}

impl NotificarLider {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_NOTIFICAR_LIDER {
            return Err("Opcode incorrecto para NotificarLider".to_string());
        }
        let mut offset = 1;
        let id_lider = read_usize(bytes, &mut offset)?;
        let id_iniciador = read_usize(bytes, &mut offset)?;
        Ok(NotificarLider {
            id_lider,
            id_iniciador,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_NOTIFICAR_LIDER);
        write_usize(&mut buf, self.id_lider);
        write_usize(&mut buf, self.id_iniciador);
        buf
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AceptarCliente {
    pub stream: TcpStream,
    pub peer_addr: SocketAddr,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CambiarConexionListener {
    pub stream: TcpStream,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HabilitarSurtidor {
    pub surtidor_id: usize,
    pub surtidor_addr: Addr<Surtidor>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InformarVenta {
    pub venta: Venta,
    pub id_surtidor: usize,
    pub id_estacion: usize,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct EnviarVentasAgrupadas;

impl InformarVenta {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_INFORMAR_VENTA {
            return Err("Opcode incorrecto para InformarVenta".to_string());
        }

        let mut offset = 1;

        // Leer venta
        let venta = read_venta(bytes, &mut offset)?;

        // Leer id_surtidor
        let id_surtidor = read_usize(bytes, &mut offset)?;

        // Leer id_estacion
        let id_estacion = read_usize(bytes, &mut offset)?;

        Ok(InformarVenta {
            venta,
            id_surtidor,
            id_estacion,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_INFORMAR_VENTA);

        // venta
        write_venta(&mut buf, &self.venta);

        // id_surtidor
        write_usize(&mut buf, self.id_surtidor);

        // id_estacion
        write_usize(&mut buf, self.id_estacion);

        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct TransaccionesPorEstacion {
    pub transacciones: HashMap<usize, HashMap<usize, Vec<(usize, bool)>>>,
}

impl TransaccionesPorEstacion {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_TRANSACCIONES_POR_ESTACION {
            return Err("Opcode incorrecto para TransaccionesPorEstacion".to_string());
        }
        let mut offset = 1;
        let cant_estaciones = read_usize(bytes, &mut offset)?;
        let mut transacciones = HashMap::new();

        for _ in 0..cant_estaciones {
            let id_estacion = read_usize(bytes, &mut offset)?;
            let cant_surtidores = read_usize(bytes, &mut offset)?;
            let mut surtidores = HashMap::new();

            for _ in 0..cant_surtidores {
                let id_surtidor = read_usize(bytes, &mut offset)?;
                let cant_transacciones = read_usize(bytes, &mut offset)?;
                let mut transacciones_vec = Vec::new();

                for _ in 0..cant_transacciones {
                    let id_venta = read_usize(bytes, &mut offset)?;
                    let ok = read_bool(bytes, &mut offset)?;
                    transacciones_vec.push((id_venta, ok));
                }

                surtidores.insert(id_surtidor, transacciones_vec);
            }

            transacciones.insert(id_estacion, surtidores);
        }

        Ok(TransaccionesPorEstacion { transacciones })
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct EnviarASiguiente {
    pub estacion_cercana_id: usize,
    pub msg: Vec<u8>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct DesconexionDetectada {
    pub estacion_id: usize,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct LiderCaido {
    pub mensaje: InformarVenta,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct NuevoLiderConectado;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ConfirmarTransacciones {
    pub transacciones: HashMap<usize, Vec<(usize, bool)>>,
}

impl ConfirmarTransacciones {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_CONFIRMAR_TRANSACCIONES {
            return Err("Opcode incorrecto para ConfirmarTransacciones".to_string());
        }
        let mut offset = 1;
        let cant_surtidores = read_usize(bytes, &mut offset)?;
        let mut transacciones = HashMap::new();

        for _ in 0..cant_surtidores {
            let id_surtidor = read_usize(bytes, &mut offset)?;
            let cant_ventas = read_usize(bytes, &mut offset)?;

            let mut ventas = Vec::new();

            for _ in 0..cant_ventas {
                let id_venta = read_usize(bytes, &mut offset)?;
                let ok = read_bool(bytes, &mut offset)?;

                ventas.push((id_venta, ok));
            }

            transacciones.insert(id_surtidor, ventas);
        }

        Ok(ConfirmarTransacciones { transacciones })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_CONFIRMAR_TRANSACCIONES);

        // cantidad_surtidores
        write_usize(&mut buf, self.transacciones.len());

        for (id_surtidor, ventas) in &self.transacciones {
            write_usize(&mut buf, *id_surtidor);
            write_usize(&mut buf, ventas.len());

            for (id_venta, ok) in ventas {
                write_usize(&mut buf, *id_venta);
                write_bool(&mut buf, *ok);
            }
        }

        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct EstacionDesconectada {
    pub estacion_id: usize,

    pub mensaje: Vec<u8>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct CobrarACliente {
    pub venta: Venta,
    pub surtidor_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcesarMensaje {
    pub bytes: Vec<u8>,
    pub _estacion_remota: usize, // opcional
}

// ===== Función helper para deserializar cualquier mensaje =====
/// Deserializa un mensaje desde bytes basándose en el opcode
pub fn deserialize_message(bytes: &[u8]) -> Result<MessageType, String> {
    if bytes.is_empty() {
        return Err("Buffer vacío".to_string());
    }
    let opcode = bytes[0];
    match opcode {
        OPCODE_REENVIAR => Reenviar::from_bytes(bytes).map(MessageType::Reenviar),
        OPCODE_ELECCION => Eleccion::from_bytes(bytes).map(MessageType::Eleccion),
        OPCODE_NOTIFICAR_LIDER => {
            NotificarLider::from_bytes(bytes).map(MessageType::NotificarLider)
        }
        OPCODE_INFORMAR_VENTA => InformarVenta::from_bytes(bytes).map(MessageType::InformarVenta),
        OPCODE_CONFIRMAR_TRANSACCIONES => {
            ConfirmarTransacciones::from_bytes(bytes).map(MessageType::ConfirmarTransacciones)
        }
        OPCODE_INFORMAR_VENTAS_OFFLINE => {
            InformarVentasOffline::from_bytes(bytes).map(MessageType::InformarVentasOffline)
        }
        OPCODE_TRANSACCIONES_POR_ESTACION => {
            TransaccionesPorEstacion::from_bytes(bytes).map(MessageType::TransaccionesPorEstacion)
        }
        _ => Err(format!("Opcode desconocido: 0x{:02x}", opcode)),
    }
}

/// Enum para representar cualquier tipo de mensaje
pub enum MessageType {
    Reenviar(Reenviar),
    Eleccion(Eleccion),
    NotificarLider(NotificarLider),
    InformarVenta(InformarVenta),
    ConfirmarTransacciones(ConfirmarTransacciones),
    IdentificarEstacion(IdentificarEstacion),
    InformarVentasOffline(InformarVentasOffline),
    TransaccionesPorEstacion(TransaccionesPorEstacion),
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct EmpezarInformarVentasOffline {}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InformarVentasOffline {
    pub id_lider: usize,
    pub ventas: HashMap<usize, HashMap<usize, Vec<Venta>>>,
}

impl InformarVentasOffline {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_INFORMAR_VENTAS_OFFLINE {
            return Err("Opcode incorrecto para InformarVentasOffline".to_string());
        }
        let mut offset = 1;
        let id_lider = read_usize(bytes, &mut offset)?;
        let cant_estaciones = read_usize(bytes, &mut offset)?;
        let mut ventas = HashMap::new();

        for _ in 0..cant_estaciones {
            let id_estacion = read_usize(bytes, &mut offset)?;
            let cant_surtidores = read_usize(bytes, &mut offset)?;
            let mut surtidores = HashMap::new();

            for _ in 0..cant_surtidores {
                let id_surtidor = read_usize(bytes, &mut offset)?;
                let cant_ventas = read_usize(bytes, &mut offset)?;
                let mut ventas_vec = Vec::new();

                for _ in 0..cant_ventas {
                    ventas_vec.push(read_venta(bytes, &mut offset)?);
                }

                surtidores.insert(id_surtidor, ventas_vec);
            }

            ventas.insert(id_estacion, surtidores);
        }

        Ok(InformarVentasOffline { id_lider, ventas })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_INFORMAR_VENTAS_OFFLINE);
        write_usize(&mut buf, self.id_lider);
        write_usize(&mut buf, self.ventas.len());

        for (id_estacion, surtidores) in &self.ventas {
            write_usize(&mut buf, *id_estacion);
            write_usize(&mut buf, surtidores.len());

            for (id_surtidor, ventas_vec) in surtidores {
                write_usize(&mut buf, *id_surtidor);
                write_usize(&mut buf, ventas_vec.len());

                for venta in ventas_vec {
                    write_venta(&mut buf, venta);
                }
            }
        }

        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct LiberarClientesEnCola;
