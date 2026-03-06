use crate::actores::ypf::EmpresaConectada;
use actix::{Addr, Message};
use tokio::net::TcpStream;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PeerDesconectado {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct IniciarEleccion;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EleccionTimeout;

#[derive(Message)]
#[rtype(result = "()")]
pub struct NuevoLider {
    pub id: usize,
}

impl NuevoLider {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 3 {
            println!("Error: bytes length incorrecto para NuevoLider");
            return NuevoLider { id: 0 };
        }
        if bytes[0] != b'4' || bytes[1] != b'+' {
            println!("Error: formato incorrecto para NuevoLider");
            return NuevoLider { id: 0 };
        }

        let id_bytes = if bytes[bytes.len() - 1] == b'\n' {
            &bytes[2..bytes.len() - 1]
        } else {
            &bytes[2..]
        };

        let id = if let Ok(id_str) = std::str::from_utf8(id_bytes) {
            if let Ok(parsed_id) = id_str.trim().parse::<usize>() {
                parsed_id
            } else {
                println!(
                    "Error: no se pudo parsear el id para NuevoLider: '{}'",
                    id_str
                );
                0
            }
        } else {
            println!("Error: no se pudo convertir bytes a UTF-8 para NuevoLider");
            0
        };
        NuevoLider { id }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![b'4', b'+'];
        result.extend(self.id.to_string().as_bytes());
        result.push(b'\n');
        result
    }
}

#[derive(Message)]
#[rtype(result = "()")]
/// Mensaje para manejar conexiones entrantes
pub struct ConexionEntrante {
    pub peer_id: usize,
    pub socket: TcpStream,
}

#[derive(Message)]
#[rtype(result = "()")]
/// Mensaje para notificar que el socket de un peer está listo
pub struct SocketListo {
    pub peer_id: usize,
}

const OPCODE_CONFIGURAR_LIMITE: u8 = 0x10;
const OPCODE_CONFIGURAR_LIMITE_GENERAL: u8 = 0x11;
const OPCODE_GASTOS_EMPRESA: u8 = 0x12;

#[derive(Message)]
#[rtype(result = "()")]
pub struct AgregarEmpresa {
    pub empresa: Addr<EmpresaConectada>,
    pub empresa_id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct EnviarBytesEmpresa {
    pub empresa_id: usize,
    pub bytes: Vec<u8>,
}

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
}

pub const OPCODE_IDENTIFICAR_EMPRESA: u8 = 0x13;

fn read_usize(buf: &[u8], offset: &mut usize) -> Result<usize, String> {
    read_u64(buf, offset).map(|v| v as usize)
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

fn read_f32(buf: &[u8], offset: &mut usize) -> Result<f32, String> {
    if *offset + 4 > buf.len() {
        return Err("Buffer insuficiente para leer f32".to_string());
    }
    let bytes = &buf[*offset..*offset + 4];
    *offset += 4;
    Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcesarMensajeEmpresa {
    pub bytes: Vec<u8>,
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
        Ok(ConfigurarLimite {
            id_tarjeta,
            id_empresa,
            monto,
        })
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
        OPCODE_CONFIGURAR_LIMITE => {
            ConfigurarLimite::from_bytes(bytes).map(MessageType::ConfigurarLimite)
        }
        OPCODE_CONFIGURAR_LIMITE_GENERAL => {
            ConfigurarLimiteGeneral::from_bytes(bytes).map(MessageType::ConfigurarLimiteGeneral)
        }
        OPCODE_GASTOS_EMPRESA => GastosEmpresa::from_bytes(bytes).map(MessageType::GastosEmpresa),
        _ => Err(format!("Opcode desconocido: 0x{:02x}", opcode)),
    }
}
