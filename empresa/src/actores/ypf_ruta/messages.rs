use actix::{Context, Handler, Message};
use crate::actores::ypf_ruta::ypf_ruta::YpfRuta;

// ===== Opcodes del protocolo =====
const OPCODE_CONFIGURAR_LIMITE: u8 = 0x10;
const OPCODE_CONFIGURAR_LIMITE_GENERAL: u8 = 0x11;
const OPCODE_GASTOS_EMPRESA: u8 = 0x12;

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

#[derive(Message)]
#[rtype(result = "()")]
pub struct Enviar {
    pub bytes: Vec<u8>,
}

impl Handler<Enviar> for YpfRuta {
    type Result = ();

    fn handle(&mut self, msg: Enviar, _ctx: &mut Context<Self>) {
        println!(
            "Enviando mensaje a YPF Ruta",
        );
        let buf = msg.bytes.clone();
        self.enviar_por_socket(buf);
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ConfigurarLimite {
    pub id_tarjeta: usize,
    pub id_empresa: usize,
    pub monto: f32,
}

impl ConfigurarLimite {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_CONFIGURAR_LIMITE {
            return Err("Opcode incorrecto para ConfigurarLimite".to_string());
        }
        let mut offset = 1;
        let id_tarjeta = read_usize(bytes, &mut offset)?;
        let id_empresa = read_usize(bytes, &mut offset)?;
        let monto = read_f32(bytes, &mut offset)?;
        Ok(ConfigurarLimite { id_tarjeta, id_empresa, monto })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_CONFIGURAR_LIMITE);
        write_usize(&mut buf, self.id_tarjeta);
        write_usize(&mut buf, self.id_empresa);
        write_f32(&mut buf, self.monto);
        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ConfigurarLimiteGeneral {
    pub id_empresa: usize,
    pub monto: f32,
}

impl ConfigurarLimiteGeneral {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_CONFIGURAR_LIMITE_GENERAL {
            return Err("Opcode incorrecto para ConfigurarLimiteGeneral".to_string());
        }
        let mut offset = 1;
        let id_empresa = read_usize(bytes, &mut offset)?;
        let monto = read_f32(bytes, &mut offset)?;
        Ok(ConfigurarLimiteGeneral { id_empresa, monto })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_CONFIGURAR_LIMITE_GENERAL);
        write_usize(&mut buf, self.id_empresa);
        write_f32(&mut buf, self.monto);
        buf
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct GastosEmpresa {
    pub id_empresa: usize,
}

impl GastosEmpresa {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_GASTOS_EMPRESA {
            return Err("Opcode incorrecto para GastosEmpresa".to_string());
        }
        let mut offset = 1;
        let id_empresa = read_usize(bytes, &mut offset)?;
        Ok(GastosEmpresa { id_empresa })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_GASTOS_EMPRESA);
        write_usize(&mut buf, self.id_empresa);
        buf
    }
}

pub enum MessageType {
    ConfigurarLimite(ConfigurarLimite),
    ConfigurarLimiteGeneral(ConfigurarLimiteGeneral),
    GastosEmpresa(GastosEmpresa),
}

// ===== Función helper para deserializar cualquier mensaje =====
/// Deserializa un mensaje desde bytes basándose en el opcode
pub fn deserialize_message(bytes: &[u8]) -> Result<MessageType, String> {
    if bytes.is_empty() {
        return Err("Buffer vacío".to_string());
    }
    let opcode = bytes[0];
    match opcode {
        OPCODE_CONFIGURAR_LIMITE => ConfigurarLimite::from_bytes(bytes).map(MessageType::ConfigurarLimite),
        OPCODE_CONFIGURAR_LIMITE_GENERAL => ConfigurarLimiteGeneral::from_bytes(bytes).map(MessageType::ConfigurarLimiteGeneral),
        OPCODE_GASTOS_EMPRESA => GastosEmpresa::from_bytes(bytes).map(MessageType::GastosEmpresa),
        _ => Err(format!("Opcode desconocido: 0x{:02x}", opcode)),
    }
}
