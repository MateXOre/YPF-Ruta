use actix::Message;
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

        // Extraer los bytes del ID (desde posición 2 hasta el final, sin el \n)
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

// Mensaje para manejar conexiones entrantes
#[derive(Message)]
#[rtype(result = "()")]
pub struct ConexionEntrante {
    pub peer_id: usize,
    pub socket: TcpStream,
}

// Mensaje para notificar que el socket de un peer está listo
#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketListo {
    pub peer_id: usize,
}
