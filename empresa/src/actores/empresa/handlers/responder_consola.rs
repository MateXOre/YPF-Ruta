use actix::{Context, Handler};
use crate::actores::empresa::Empresa;
use crate::actores::empresa::messages::ResponderConsola;
use crate::actores::ypf_ruta::messages::{Enviar, ConfigurarLimite, ConfigurarLimiteGeneral, GastosEmpresa};

impl Handler<ResponderConsola> for Empresa {
    type Result = ();
    fn handle(&mut self, msg: ResponderConsola, _: &mut Context<Self>) {
        let (operacion, parametros) = msg.linea.split_once(' ').unwrap_or((&msg.linea, ""));
        let parametros_de_operacion = parametros.split(',').collect::<Vec<&str>>();

        if let Some(ypf_ruta_addr) = &self.ypf_ruta_addr {
            match operacion {
                "configurar_limite" => {
                    let id_tarjeta = parametros_de_operacion[0].parse::<usize>().unwrap();
                    let monto = parametros_de_operacion[1].parse::<f32>().unwrap();
                    ypf_ruta_addr.do_send(Enviar{bytes: ConfigurarLimite { id_tarjeta, id_empresa: self.id, monto }.to_bytes()});
                }
                "configurar_limite_general" => {
                    let monto = parametros_de_operacion[0].parse::<f32>().unwrap();
                    let id_empresa = self.id;
                    ypf_ruta_addr.do_send(Enviar{bytes: ConfigurarLimiteGeneral { id_empresa, monto }.to_bytes()});
                }
                "gastos_empresa" => {
                    let id_empresa = self.id;
                    ypf_ruta_addr.do_send(Enviar{bytes: GastosEmpresa { id_empresa }.to_bytes()});
                }
                _ => {
                    println!("[Empresa {}] Operación no válida: {}", self.id, operacion);
                }
            }
        }

        println!("[Empresa {}] Entrada recibida: {}", self.id, msg.linea);

    }
}