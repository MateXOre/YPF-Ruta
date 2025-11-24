use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use serde_json;

use actix::prelude::*;
use util::structs::venta::Venta;
use crate::actores::gestor::structs::{Empresa, Tarjeta};

const PERSISTENCE_INTERVAL_SECS: u64 = 30;

pub struct Gestor {
    empresas: HashMap<usize, Empresa>,
    tarjetas: HashMap<usize, Tarjeta>,
    ventas: Vec<Venta>,
}

impl Default for Gestor {
    fn default() -> Self {
        Self {
            empresas: HashMap::new(),
            tarjetas: HashMap::new(),
            ventas: Vec::new(),
        }
    }
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
    pub fn new() -> Gestor {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("data");

        let empresas_path = data_dir.join("empresas.json");
        let tarjetas_path = data_dir.join("tarjetas.json");
        let ventas_path = data_dir.join("ventas.json");

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
        }
    }

    pub fn modificar_limite_general_empresa(&mut self, id_empresa: usize, nuevo_limite: f32) -> Result<(), String> {
        if let Some(empresa) = self.empresas.get_mut(&id_empresa) {
            if empresa.consumo_actual > nuevo_limite {
                let msg = format!(
                    "No se puede establecer el nuevo límite general {} para la empresa con ID {} ya que el consumo actual {} lo supera",
                    nuevo_limite, id_empresa, empresa.consumo_actual
                );
                eprintln!("{}", msg);
                return Err(msg);
            }
            empresa.limite_general = nuevo_limite as u64;
            Ok(())
        } else {
            let msg = format!("Empresa con ID {} no encontrada", id_empresa);
            eprintln!("{}", msg);
            Err(msg)
        }
    }

    pub fn modificar_limite_particular_tarjeta(&mut self, id_tarjeta: usize, nuevo_limite: f32) -> Result<(), String> {
        if let Some(tarjeta) = self.tarjetas.get_mut(&id_tarjeta) {
            if tarjeta.consumo_actual > nuevo_limite {
                let msg = format!(
                    "No se puede establecer el nuevo límite particular {} para la tarjeta con ID {} ya que el consumo actual {} lo supera",
                    nuevo_limite, id_tarjeta, tarjeta.consumo_actual
                );
                eprintln!("{}", msg);
                return Err(msg);
            }
            tarjeta.limite_particular = nuevo_limite as u64;
            Ok(())
        } else {
            let msg = format!("Tarjeta con ID {} no encontrada", id_tarjeta);
            eprintln!("{}", msg);
            Err(msg)
        }
    }

    pub fn consultar_estado_empresa_internal(&self, id_empresa: usize) -> Option<(Empresa, Vec<Tarjeta>)> {
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
        if let Some(t) = self.tarjetas.get_mut(&venta.id_tarjeta) {
            t.consumo_actual += venta.monto;
            let empresa_id = t.empresa_id;
            if let Some(e) = self.empresas.get_mut(&empresa_id) {
                e.consumo_actual += venta.monto;
            }
        }
        self.ventas.push(venta);
    }

    pub fn procesar_venta_internal(&mut self, venta: &Venta) -> bool {
        let tarjeta = match self.tarjetas.get(&venta.id_tarjeta) {
            Some(t) => t.clone(),
            None => {
                eprintln!("Tarjeta con ID {} no encontrada", venta.id_tarjeta);
                return false;
            }
        };

        let empresa = match self.empresas.get(&tarjeta.empresa_id) {
            Some(e) => e.clone(),
            None => {
                eprintln!("Empresa con ID {} no encontrada", tarjeta.empresa_id);
                return false;
            }
        };

        if tarjeta.consumo_actual + venta.monto > tarjeta.limite_particular as f32 {
            eprintln!(
                "Venta rechazada: excede el límite particular de la tarjeta (ID {})",
                tarjeta.id
            );
            return false;
        }

        if empresa.consumo_actual + venta.monto > empresa.limite_general as f32 {
            eprintln!(
                "Venta rechazada: excede el límite general de la empresa (ID {})",
                empresa.id
            );
            return false;
        }

        self.crear_venta(venta.clone());
        true
    }

    pub fn persistir_estado_actual(&self) {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("data");

        let empresas_path = data_dir.join("empresas.json");
        let tarjetas_path = data_dir.join("tarjetas.json");
        let ventas_path = data_dir.join("ventas.json");

        // Persistir empresas
        if let Ok(file) = File::create(&empresas_path) {
            if let Err(e) = serde_json::to_writer_pretty(file, &self.empresas.values().collect::<Vec<&Empresa>>()) {
                eprintln!("Error al guardar empresas en {}: {}", empresas_path.display(), e);
            }
        } else {
            eprintln!("No se pudo crear el archivo {}", empresas_path.display());
        }

        // Persistir tarjetas
        if let Ok(file) = File::create(&tarjetas_path) {
            if let Err(e) = serde_json::to_writer_pretty(file, &self.tarjetas.values().collect::<Vec<&Tarjeta>>()) {
                eprintln!("Error al guardar tarjetas en {}: {}", tarjetas_path.display(), e);
            }
        } else {
            eprintln!("No se pudo crear el archivo {}", tarjetas_path.display());
        }

        // Persistir ventas
        if let Ok(file) = File::create(&ventas_path) {
            if let Err(e) = serde_json::to_writer_pretty(file, &self.ventas) {
                eprintln!("Error al guardar ventas en {}: {}", ventas_path.display(), e);
            }
        } else {
            eprintln!("No se pudo crear el archivo {}", ventas_path.display());
        }
    }
}

impl Actor for Gestor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Persistir regularmente cada 30 segundos 
        ctx.run_interval(Duration::from_secs(PERSISTENCE_INTERVAL_SECS), |act, _ctx| {
            act.persistir_estado_actual();
            println!("Gestor: estado persistido periódicamente");
        });

        println!("Gestor actor iniciado");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Gestor actor detenido; persistiendo estado final");
        self.persistir_estado_actual();
    }
}
