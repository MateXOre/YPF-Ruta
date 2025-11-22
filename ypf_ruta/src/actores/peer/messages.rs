use actix::Message;
use chrono::Local;
use tokio::net::TcpStream;
use crate::actores::gestor::structs::Venta;

#[derive(Message)]
#[rtype(result = "()")]
pub struct GuardarSocket(pub(crate) TcpStream);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Eleccion(pub(crate) usize);

impl Eleccion {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 3 {
            println!("Error: bytes length incorrecto para Eleccion");
            return Eleccion(0);
        }
        if bytes[0] != b'3' || bytes[1] != b'+' {
            println!("Error: formato incorrecto para Eleccion");
            return Eleccion(0);
        }
        let id_bytes = &bytes[2..];
        let id = std::str::from_utf8(id_bytes).unwrap_or("0").parse::<usize>().unwrap_or(0);
        Eleccion(id)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![b'3', b'+'];
        result.extend(self.0.to_string().as_bytes());
        result
    }
}

// Respuesta OK a una elección (usado en algoritmo Bully)
#[derive(Message)]
#[rtype(result = "()")]
pub struct EleccionOk(pub(crate) usize);

impl EleccionOk {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 3 {
            println!("Error: bytes length incorrecto para EleccionOk");
            return EleccionOk(0);
        }
        if bytes[0] != b'6' || bytes[1] != b'+' {
            println!("Error: formato incorrecto para EleccionOk");
            return EleccionOk(0);
        }
        let id_bytes = &bytes[2..];
        let id = std::str::from_utf8(id_bytes).unwrap_or("0").parse::<usize>().unwrap_or(0);
        EleccionOk(id)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![b'6', b'+'];
        result.extend(self.0.to_string().as_bytes());
        result
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct VentaRegistrada {
    pub venta: Venta,
}

impl VentaRegistrada {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes[0] != b'5' || bytes[1] != b'+' {
            println!("Error: formato incorrecto para VentaRegistrada");
            return VentaRegistrada { venta: Venta {
                id: 0,
                id_estacion: 0,
                id_tarjeta: 0,
                monto: 0,
                fecha: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            } };
        }

        let parts: Vec<&str> = std::str::from_utf8(bytes).unwrap_or("").split(',').collect();
        if parts.len() != 5 {
            println!("Error: formato incorrecto para VentaRegistrada");
            return VentaRegistrada { venta: Venta {
                id: 0,
                id_estacion: 0,
                id_tarjeta: 0,
                monto: 0,
                fecha: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            } };

        }


        let id = parts[0].parse::<u64>().unwrap_or(0);
        let id_estacion = parts[1].parse::<u64>().unwrap_or(0);
        let id_tarjeta = parts[2].parse::<u64>().unwrap_or(0);
        let monto = parts[3].parse::<u64>().unwrap_or(0);
        let fecha = parts[4].to_string();

        VentaRegistrada {
            venta: Venta {
                id,
                id_estacion,
                id_tarjeta,
                monto,
                fecha,
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        format!("5+{},{},{},{},{:?}", self.venta.id, self.venta.id_estacion, self.venta.id_tarjeta, self.venta.monto, self.venta.fecha).as_bytes().to_vec()
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcesarMensaje {
    pub bytes: Vec<u8>
}
