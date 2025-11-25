use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::actores::gestor::structs::{Empresa, Tarjeta};
use actix::prelude::*;
use util::{
    log_error, log_info, log_warning,
    structs::venta::{EstadoVenta, Venta},
};

const PERSISTENCE_INTERVAL_SECS: u64 = 30;

pub struct Gestor {
    empresas: HashMap<usize, Empresa>,
    tarjetas: HashMap<usize, Tarjeta>,
    ventas: Vec<Venta>,
    index: usize,
    logger: Sender<Vec<u8>>,
}

fn load_json_vec<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Vec<T> {
    match File::open(path) {
        Ok(f) => match serde_json::from_reader(f) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("No se pudo parsear {}: {}", path.display(), e);
                Vec::new()
            }
        },
        Err(e) => {
            eprintln!("No se pudo abrir {}: {}", path.display(), e);
            Vec::new()
        }
    }
}

impl Gestor {
    pub fn new(index: usize, logger: Sender<Vec<u8>>) -> Gestor {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("data");

        let empresas_path = data_dir.join(format!("empresas_{}.json", index));
        let tarjetas_path = data_dir.join(format!("tarjetas_{}.json", index));
        let ventas_path = data_dir.join(format!("ventas_{}.json", index));

        let empresas_vec: Vec<Empresa> = load_json_vec(&empresas_path);
        let tarjetas_vec: Vec<Tarjeta> = load_json_vec(&tarjetas_path);
        let ventas_vec: Vec<Venta> = load_json_vec(&ventas_path);

        let mut empresas_map: HashMap<usize, Empresa> = HashMap::new();
        for e in empresas_vec {
            empresas_map.insert(e.id, e);
        }

        let mut tarjetas_map: HashMap<usize, Tarjeta> = HashMap::new();
        for t in tarjetas_vec {
            tarjetas_map.insert(t.id, t);
        }

        Self {
            empresas: empresas_map,
            tarjetas: tarjetas_map,
            ventas: ventas_vec,
            index,
            logger,
        }
    }

    pub fn modificar_limite_general_empresa(
        &mut self,
        id_empresa: usize,
        nuevo_limite: f32,
    ) -> Result<(), String> {
        if let Some(empresa) = self.empresas.get_mut(&id_empresa) {
            if empresa.consumo_actual > nuevo_limite {
                let msg = format!(
                    "No se puede establecer el nuevo límite general {} para la empresa con ID {} ya que el consumo actual {} lo supera",
                    nuevo_limite, id_empresa, empresa.consumo_actual
                );
                log_warning!(self.logger, "{}", msg);
                return Err(msg);
            }
            empresa.limite_general = nuevo_limite as u64;
            Ok(())
        } else {
            let msg = format!("Empresa con ID {} no encontrada", id_empresa);
            log_warning!(self.logger, "{}", msg);
            Err(msg)
        }
    }

    pub fn modificar_limite_particular_tarjeta(
        &mut self,
        id_tarjeta: usize,
        nuevo_limite: f32,
    ) -> Result<(), String> {
        if let Some(tarjeta) = self.tarjetas.get_mut(&id_tarjeta) {
            if tarjeta.consumo_actual > nuevo_limite {
                let msg = format!(
                    "No se puede establecer el nuevo límite particular {} para la tarjeta con ID {} ya que el consumo actual {} lo supera",
                    nuevo_limite, id_tarjeta, tarjeta.consumo_actual
                );
                log_warning!(self.logger, "{}", msg);
                return Err(msg);
            }
            tarjeta.limite_particular = nuevo_limite as u64;
            Ok(())
        } else {
            let msg = format!("Tarjeta con ID {} no encontrada", id_tarjeta);
            log_warning!(self.logger, "{}", msg);
            Err(msg)
        }
    }

    pub fn consultar_estado_empresa_internal(
        &self,
        id_empresa: usize,
    ) -> Option<(Empresa, Vec<Tarjeta>)> {
        self.empresas.get(&id_empresa).map(|empresa| {
            let tarjetas: Vec<Tarjeta> = self
                .tarjetas
                .values()
                .filter(|t| t.empresa_id == id_empresa)
                .cloned()
                .collect();
            (empresa.clone(), tarjetas)
        })
    }

    pub fn crear_venta(&mut self, venta: Venta) {
        if let Some(nueva_venta) = self.ventas.iter().find(|v| v.id_venta == venta.id_venta) {
            log_info!(
                self.logger,
                "Venta con ID {} ya procesada previamente",
                nueva_venta.id_venta
            );
            return;
        }
        if let Some(t) = self.tarjetas.get_mut(&venta.id_tarjeta) {
            let mut consumo = venta.monto;
            if t.consumo_actual + consumo > t.limite_particular as f32 {
                consumo = (t.limite_particular as f32) - (t.consumo_actual);
                t.consumo_actual = t.limite_particular as f32;
            } else {
                t.consumo_actual += consumo;
            }
            let empresa_id = t.empresa_id;
            if let Some(e) = self.empresas.get_mut(&empresa_id) {
                if e.consumo_actual + consumo > e.limite_general as f32 {
                    e.consumo_actual = e.limite_general as f32;
                } else {
                    e.consumo_actual += consumo;
                }
            }
        }
        self.ventas.push(venta);
    }

    pub fn procesar_venta_internal(&mut self, venta: &Venta) -> EstadoVenta {
        let mut estado = EstadoVenta::Confirmada;

        if let Some(nueva_venta) = self.ventas.iter().find(|v| v.id_venta == venta.id_venta) {
            log_info!(
                self.logger,
                "Venta con ID {} ya procesada previamente",
                nueva_venta.id_venta
            );
            return estado;
        }
        let tarjeta = match self.tarjetas.get(&venta.id_tarjeta) {
            Some(t) => t.clone(),
            None => {
                log_warning!(
                    self.logger,
                    "Tarjeta con ID {} no encontrada",
                    venta.id_tarjeta
                );
                return EstadoVenta::Rechazada;
            }
        };
        let empresa = match self.empresas.get(&tarjeta.empresa_id) {
            Some(e) => e.clone(),
            None => {
                log_warning!(
                    self.logger,
                    "Empresa con ID {} no encontrada",
                    tarjeta.empresa_id
                );
                return EstadoVenta::Rechazada;
            }
        };

        if tarjeta.consumo_actual + venta.monto > tarjeta.limite_particular as f32 {
            if !venta.offline {
                log_warning!(
                    self.logger,
                    "Venta rechazada: excede el límite particular de la tarjeta (ID {})",
                    tarjeta.id
                );
                return EstadoVenta::Rechazada;
            } else {
                estado = EstadoVenta::Rechazada;
            }
        }

        if empresa.consumo_actual + venta.monto > empresa.limite_general as f32 {
            if !venta.offline {
                log_warning!(
                    self.logger,
                    "Venta rechazada: excede el límite general de la empresa (ID {})",
                    empresa.id
                );
                return EstadoVenta::Rechazada;
            } else {
                estado = EstadoVenta::Rechazada;
            }
        }
        let venta = Venta {
            id_venta: venta.id_venta,
            id_tarjeta: venta.id_tarjeta,
            id_estacion: venta.id_estacion,
            monto: venta.monto,
            offline: venta.offline,
            estado: estado.clone(),
        };
        self.crear_venta(venta);
        estado
    }

    pub fn persistir_estado_actual(&self) {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("data");

        let empresas_path = data_dir.join(format!("empresas_{}.json", self.index));
        let tarjetas_path = data_dir.join(format!("tarjetas_{}.json", self.index));
        let ventas_path = data_dir.join(format!("ventas_{}.json", self.index));

        if let Ok(file) = File::create(&empresas_path) {
            if let Err(e) = serde_json::to_writer_pretty(
                file,
                &self.empresas.values().collect::<Vec<&Empresa>>(),
            ) {
                log_error!(
                    self.logger,
                    "Error al guardar empresas en {}: {}",
                    empresas_path.display(),
                    e
                );
            }
        } else {
            log_error!(
                self.logger,
                "No se pudo crear el archivo {}",
                empresas_path.display()
            );
        }

        if let Ok(file) = File::create(&tarjetas_path) {
            if let Err(e) = serde_json::to_writer_pretty(
                file,
                &self.tarjetas.values().collect::<Vec<&Tarjeta>>(),
            ) {
                log_error!(
                    self.logger,
                    "Error al guardar tarjetas en {}: {}",
                    tarjetas_path.display(),
                    e
                );
            }
        } else {
            log_error!(
                self.logger,
                "No se pudo crear el archivo {}",
                tarjetas_path.display()
            );
        }

        if let Ok(file) = File::create(&ventas_path) {
            if let Err(e) = serde_json::to_writer_pretty(file, &self.ventas) {
                log_error!(
                    self.logger,
                    "Error al guardar ventas en {}: {}",
                    ventas_path.display(),
                    e
                );
            }
        } else {
            log_error!(
                self.logger,
                "No se pudo crear el archivo {}",
                ventas_path.display()
            );
        }
    }
}

impl Actor for Gestor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(
            Duration::from_secs(PERSISTENCE_INTERVAL_SECS),
            |act, _ctx| {
                act.persistir_estado_actual();
                log_info!(act.logger, "Gestor: estado persistido periódicamente");
            },
        );

        log_info!(self.logger, "Gestor actor iniciado");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log_info!(
            self.logger,
            "Gestor actor detenido; persistiendo estado final"
        );
        self.persistir_estado_actual();
    }
}
