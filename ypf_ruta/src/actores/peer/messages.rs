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

        // Extraer los bytes del ID (desde posición 2 hasta el final, sin el \n)
        let id_bytes = if bytes[bytes.len() - 1] == b'\n' {
            &bytes[2..bytes.len() - 1]
        } else {
            &bytes[2..]
        };

        let id = if let Ok(id_str) = std::str::from_utf8(id_bytes) {
            if let Ok(id) = id_str.trim().parse::<usize>() {
                id
            } else {
                println!(
                    "Error: no se pudo parsear el id para Eleccion: '{}'",
                    id_str
                );
                0
            }
        } else {
            println!("Error: no se pudo convertir bytes a UTF-8 para Eleccion");
            0
        };
        Eleccion(id)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![b'3', b'+'];
        result.extend(self.0.to_string().as_bytes());
        result.push(b'\n');
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

        // Extraer los bytes del ID (desde posición 2 hasta el final, sin el \n)
        let id_bytes = if bytes[bytes.len() - 1] == b'\n' {
            &bytes[2..bytes.len() - 1]
        } else {
            &bytes[2..]
        };

        let id = if let Ok(id_str) = std::str::from_utf8(id_bytes) {
            if let Ok(id) = id_str.trim().parse::<usize>() {
                id
            } else {
                println!(
                    "Error: no se pudo parsear el id para EleccionOk: '{}'",
                    id_str
                );
                0
            }
        } else {
            println!("Error: no se pudo convertir bytes a UTF-8 para EleccionOk");
            0
        };
        EleccionOk(id)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![b'6', b'+'];
        result.extend(self.0.to_string().as_bytes());
        result.push(b'\n');
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
        // Formato: b'5' + JSON de Venta + b'\n'
        if bytes.len() < 3 || bytes[0] != b'5' {
            println!("Error: formato incorrecto para VentaRegistrada (falta prefijo b'5')");
            return VentaRegistrada {
                venta: Venta {
                    id_venta: 0,
                    id_estacion: 0,
                    id_tarjeta: 0,
                    monto: 0.0,
                    offline: false,
                    estado: util::structs::venta::EstadoVenta::Rechazada,
                },
            };
        }

        // Extraer el JSON (desde posición 1 hasta el final, ignorando el \n final si existe)
        let json_bytes = if bytes[bytes.len() - 1] == b'\n' {
            &bytes[1..bytes.len() - 1]
        } else {
            &bytes[1..]
        };

        // Deserializar el JSON
        match serde_json::from_slice::<Venta>(json_bytes) {
            Ok(venta) => VentaRegistrada { venta },
            Err(e) => {
                eprintln!("Error deserializando VentaRegistrada: {:?}", e);
                eprintln!("Bytes recibidos: {:?}", String::from_utf8_lossy(json_bytes));
                VentaRegistrada {
                    venta: Venta {
                        id_venta: 0,
                        id_estacion: 0,
                        id_tarjeta: 0,
                        monto: 0.0,
                        offline: false,
                        estado: util::structs::venta::EstadoVenta::Rechazada,
                    },
                }
            }
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        // Formato: b'5' + JSON de Venta + b'\n'
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
    pub bytes: Vec<u8>,
}
