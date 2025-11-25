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

fn write_f32(buf: &mut Vec<u8>, value: f32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn write_usize(buf: &mut Vec<u8>, value: usize) {
    write_u64(buf, value as u64);
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
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(OPCODE_GASTOS_EMPRESA);
        write_usize(&mut buf, self.id_empresa);
        buf
    }
}
