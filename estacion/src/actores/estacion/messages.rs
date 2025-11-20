use actix::{Message, Addr};
use std::net::SocketAddr;
use crate::actores::{estacion_cercana::EstacionCercana, surtidor::surtidor::Surtidor};
use util::structs::venta::Venta;

// ===== Opcodes del protocolo =====
const OPCODE_REENVIAR: u8 = 0x01;
const OPCODE_ELECCION: u8 = 0x02;
const OPCODE_NOTIFICAR_LIDER: u8 = 0x03;
const OPCODE_INFORMAR_VENTA: u8 = 0x04;
const OPCODE_CONFIRMAR_TRANSACCIONES: u8 = 0x05;

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
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
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

fn write_string(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    write_u32(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

fn read_string(buf: &[u8], offset: &mut usize) -> Result<String, String> {
    let len = read_u32(buf, offset)? as usize;
    if *offset + len > buf.len() {
        return Err("Buffer insuficiente para leer string".to_string());
    }
    let bytes = &buf[*offset..*offset + len];
    *offset += len;
    String::from_utf8(bytes.to_vec())
        .map_err(|e| format!("Error decodificando UTF-8: {}", e))
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
        util::structs::venta::EstadoVenta::Fallida => 2u8,
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
        2 => util::structs::venta::EstadoVenta::Fallida,
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
pub struct AgregarEstacion {
    pub estacion: Addr<EstacionCercana>,
    pub estacion_id: usize,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Reenviar(pub String);

impl Reenviar {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_REENVIAR {
            return Err("Opcode incorrecto para Reenviar".to_string());
        }
        let mut offset = 1;
        let string = read_string(bytes, &mut offset)?;
        Ok(Reenviar(string))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_REENVIAR);
        write_string(&mut buf, &self.0);
        buf
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
        Ok(NotificarLider { id_lider, id_iniciador })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_NOTIFICAR_LIDER);
        write_usize(&mut buf, self.id_lider);
        write_usize(&mut buf, self.id_iniciador);
        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct HabilitarSurtidor {
    pub surtidor_id: usize,
    pub surtidor_addr: Addr<Surtidor>,
}


#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct InformarVenta {
    pub venta: Venta
}

impl InformarVenta {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_INFORMAR_VENTA {
            return Err("Opcode incorrecto para InformarVenta".to_string());
        }
        let mut offset = 1;
        let venta = read_venta(bytes, &mut offset)?;
        Ok(InformarVenta { venta })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_INFORMAR_VENTA);
        write_venta(&mut buf, &self.venta);
        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct TransaccionesPorEstacion {
    pub transacciones: Vec<Venta>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ConfirmarTransacciones {
    pub transacciones: Vec<Venta>,
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
        let count = read_u32(bytes, &mut offset)? as usize;
        let mut transacciones = Vec::with_capacity(count);
        for _ in 0..count {
            transacciones.push(read_venta(bytes, &mut offset)?);
        }
        Ok(ConfirmarTransacciones { transacciones })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_CONFIRMAR_TRANSACCIONES);
        write_u32(&mut buf, self.transacciones.len() as u32);
        for venta in &self.transacciones {
            write_venta(&mut buf, venta);
        }
        buf
    }
}


#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct CobrarACliente {
    pub venta: Venta,
    pub surtidor_id: usize,
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
        OPCODE_NOTIFICAR_LIDER => NotificarLider::from_bytes(bytes).map(MessageType::NotificarLider),
        OPCODE_INFORMAR_VENTA => InformarVenta::from_bytes(bytes).map(MessageType::InformarVenta),
        OPCODE_CONFIRMAR_TRANSACCIONES => ConfirmarTransacciones::from_bytes(bytes).map(MessageType::ConfirmarTransacciones),
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
}

