use crate::actores::gestor::structs::{Empresa, Tarjeta};
use actix::Message;
use util::structs::venta::Venta;

#[derive(Message)]
#[rtype(result = "bool")]
pub struct ValidarVenta(pub Venta);

#[derive(Message)]
#[rtype(result = "Option<(Empresa, Vec<Tarjeta>)>")]
pub struct ConsultarEstado(pub usize);

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct ModificarLimiteGeneral {
    pub id_empresa: usize,
    pub nuevo_limite: u64,
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct ModificarLimiteParticular {
    pub id_tarjeta: usize,
    pub nuevo_limite: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PersistirEstado;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegistrarVenta(pub Venta);
