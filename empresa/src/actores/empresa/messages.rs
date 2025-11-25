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


fn write_usize(buf: &mut Vec<u8>, value: usize) {
    write_u64(buf, value as u64);
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
    pub mensaje: Option<String>,
    pub empresa: Option<Value>,
    pub tarjetas: Option<Value>,
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
        .ok_or_else(|| "Campo 'tipo' no encontrado o invÃ¡lido".to_string())?;

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
                mensaje: json.get("mensaje").and_then(|v| v.as_str()).map(|s| s.to_string()),
                empresa: json.get("empresa").cloned(),
                tarjetas: json.get("tarjetas").cloned(),
            };
            Ok(RespuestaYpfRuta::GastosEmpresa(respuesta))
        }
        _ => Err(format!("Tipo de mensaje desconocido: {}", tipo)),
    }
}

