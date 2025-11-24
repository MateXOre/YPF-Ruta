use actix::Message;
use tokio::net::TcpStream;
use util::structs::venta::Venta;

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
                id_venta: 0,
                id_estacion: 0,
                id_tarjeta: 0,
                monto: 0.0,
                offline: false,
                estado: util::structs::venta::EstadoVenta::Fallida
            } };
        }

        let parts: Vec<&str> = std::str::from_utf8(bytes).unwrap_or("").split(',').collect();
        if parts.len() != 5 {
            println!("Error: formato incorrecto para VentaRegistrada");
            return VentaRegistrada { venta: Venta {
                id_estacion: 0,
                id_tarjeta: 0,
                monto: 0.0,
                id_venta: 0,
                offline: false,
                estado: util::structs::venta::EstadoVenta::Fallida,
            } };

        }


        let id = parts[0].parse::<usize>().unwrap_or(0);
        let id_estacion = parts[1].parse::<usize>().unwrap_or(0);
        let id_tarjeta = parts[2].parse::<usize>().unwrap_or(0);
        let monto = parts[3].parse::<f32>().unwrap_or(0.0);

        VentaRegistrada {
            venta: Venta {
                id_venta: id,
                id_estacion,
                id_tarjeta,
                monto,
                offline: false,
                estado: util::structs::venta::EstadoVenta::Pendiente
            }
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error>{
        let mut bytes = vec![b'5'];
        let venta_bytes = serde_json::to_vec(&self.venta)?;
        bytes.extend(venta_bytes);
        bytes.push(b'\n');
        Ok(bytes)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProcesarMensaje {
    pub bytes: Vec<u8>
}
