use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PeerDesconectado {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NuevoLider {
    pub id: usize,
}

impl NuevoLider {
    pub fn fromBytes(bytes: &[u8]) -> Self {
        if bytes.len() != 3 {
            println!("Error: bytes length incorrecto para NuevoLider");
            return NuevoLider { id: 0 };
        }
        if bytes[0] != b'4' || bytes[1] != b'+' {
            println!("Error: formato incorrecto para NuevoLider");
            return NuevoLider { id: 0 };
        }
        let id_bytes = &bytes[2..];
        let id = std::str::from_utf8(id_bytes).unwrap_or("0").parse::<usize>().unwrap_or(0);
        NuevoLider { id }
    }

    pub fn toBytes(&self) -> Vec<u8> {
        let mut result = vec![b'4', b'+'];
        result.extend(self.id.to_string().as_bytes());
        result
    }
}
