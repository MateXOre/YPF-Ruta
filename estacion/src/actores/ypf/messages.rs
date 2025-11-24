use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EnviarYpf {
    pub bytes: Vec<u8>,
}

