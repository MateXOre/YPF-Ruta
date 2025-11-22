use actix::Message;
use crate::actores::gestor::structs::{Empresa, Tarjeta, Venta};

#[derive(Message)]
#[rtype(result = "bool")]
pub struct ValidarVenta(pub Venta);

#[derive(Message)]
#[rtype(result = "Option<(Empresa, Vec<Tarjeta>)>")]
pub struct ConsultarEstado(pub u64);

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct ModificarLimiteGeneral {
    pub id_empresa: u64,
    pub nuevo_limite: u64,
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct ModificarLimiteParticular {
    pub id_tarjeta: u64,
    pub nuevo_limite: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PersistirEstado;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegistrarVenta(pub Venta);