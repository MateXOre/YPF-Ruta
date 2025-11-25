use actix::{Addr, Message};
use crate::actores::ypf_ruta::YpfRuta;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ===== Opcode del protocolo =====
pub const OPCODE_IDENTIFICAR_EMPRESA: u8 = 0x13;

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

fn write_usize(buf: &mut Vec<u8>, value: usize) {
    write_u64(buf, value as u64);
}

fn read_usize(buf: &[u8], offset: &mut usize) -> Result<usize, String> {
    read_u64(buf, offset).map(|v| v as usize)
}

// Mensaje para procesar entrada de consola
#[derive(Message)]
#[rtype(result = "()")]
pub struct ResponderConsola {
    pub linea: String,
}

// Mensaje para procesar datos recibidos del socket
#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcesarMensajeSocket {
    pub bytes: Vec<u8>,
}

#[derive(Message)]
#[rtype(result="()")]
pub struct ConectadoAypfRuta {
    pub addr: Addr<YpfRuta>,
}

#[derive(Clone)]
pub struct IdentificarEmpresa {
    pub id: usize,
}

impl IdentificarEmpresa {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Buffer vacío".to_string());
        }
        if bytes[0] != OPCODE_IDENTIFICAR_EMPRESA {
            return Err("Opcode incorrecto para IdentificarEmpresa".to_string());
        }
        let mut offset = 1;
        let id = read_usize(bytes, &mut offset)?;
        Ok(IdentificarEmpresa { id })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_IDENTIFICAR_EMPRESA);
        write_usize(&mut buf, self.id);
        buf
    }
}

// ===== Mensajes de respuesta desde YpfRuta =====

/// Respuesta de ConfigurarLimite
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct RespuestaConfigurarLimite {
    pub exito: bool,
    pub mensaje: String,
    pub id_tarjeta: usize,
    pub id_empresa: usize,
}

/// Respuesta de ConfigurarLimiteGeneral
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct RespuestaConfigurarLimiteGeneral {
    pub exito: bool,
    pub mensaje: String,
    pub id_empresa: usize,
}

/// Respuesta de GastosEmpresa
#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct RespuestaGastosEmpresa {
    pub exito: bool,
    pub mensaje: String,
    pub id_empresa: usize,
    pub data: Value,
}

/// Enum con todos los tipos de respuesta posibles de YpfRuta
pub enum RespuestaYpfRuta {
    ConfigurarLimite(RespuestaConfigurarLimite),
    ConfigurarLimiteGeneral(RespuestaConfigurarLimiteGeneral),
    GastosEmpresa(RespuestaGastosEmpresa),
}

/// Deserializa bytes JSON a un tipo de respuesta de YpfRuta
pub fn deserialize_respuesta_ypfruta(bytes: &[u8]) -> Result<RespuestaYpfRuta, String> {
    let json: Value = serde_json::from_slice(bytes)
        .map_err(|e| format!("Error parseando JSON: {}", e))?;
    
    let tipo = json.get("tipo")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Campo 'tipo' no encontrado o inválido".to_string())?;
    
    match tipo {
        "ConfigurarLimite" => {
            let respuesta = RespuestaConfigurarLimite {
                exito: json.get("exito").and_then(|v| v.as_bool()).unwrap_or(false),
                mensaje: json.get("mensaje").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                id_tarjeta: json.get("id_tarjeta").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                id_empresa: json.get("id_empresa").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            };
            Ok(RespuestaYpfRuta::ConfigurarLimite(respuesta))
        }
        "ConfigurarLimiteGeneral" => {
            let respuesta = RespuestaConfigurarLimiteGeneral {
                exito: json.get("exito").and_then(|v| v.as_bool()).unwrap_or(false),
                mensaje: json.get("mensaje").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                id_empresa: json.get("id_empresa").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            };
            Ok(RespuestaYpfRuta::ConfigurarLimiteGeneral(respuesta))
        }
        "GastosEmpresa" => {
            let respuesta = RespuestaGastosEmpresa {
                exito: json.get("exito").and_then(|v| v.as_bool()).unwrap_or(false),
                mensaje: json.get("mensaje").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                id_empresa: json.get("id_empresa").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                data: json.get("data").cloned().unwrap_or(Value::Null),
            };
            Ok(RespuestaYpfRuta::GastosEmpresa(respuesta))
        }
        _ => Err(format!("Tipo de mensaje desconocido: {}", tipo)),
    }
}

