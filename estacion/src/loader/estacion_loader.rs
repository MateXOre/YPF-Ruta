use crate::errors::error::Error;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use util::structs::venta::Venta;

pub struct EstacionLoader {
    estacion_id: usize,
}

impl EstacionLoader {
    pub fn new(estacion_id: usize) -> Self {
        Self { estacion_id }
    }

    pub fn load_ventas_sin_informar(
        &self,
    ) -> Result<HashMap<usize, HashMap<usize, Vec<Venta>>>, Error> {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("storage");

        let ventas_sin_informar_path =
            data_dir.join(format!("ventas_sin_informar_{}.json", self.estacion_id));

        let ventas_sin_informar: HashMap<usize, HashMap<usize, Vec<Venta>>> =
            self.load_json_nested_hashmap(&ventas_sin_informar_path)?;
        Ok(ventas_sin_informar)
    }

    pub fn save_ventas_sin_informar(
        &self,
        ventas: &HashMap<usize, HashMap<usize, Vec<Venta>>>,
    ) -> Result<(), Error> {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("storage");

        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            return Err(Error::ErrorString(format!(
                "No se pudo crear el directorio {}: {}",
                data_dir.display(),
                e
            )));
        }

        let ventas_sin_informar_path =
            data_dir.join(format!("ventas_sin_informar_{}.json", self.estacion_id));

        self.save_json_nested_hashmap(&ventas_sin_informar_path, ventas)
    }

    pub fn clear_ventas_sin_informar(&self) -> Result<(), Error> {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("storage");

        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            return Err(Error::ErrorString(format!(
                "No se pudo crear el directorio {}: {}",
                data_dir.display(),
                e
            )));
        }

        let ventas_sin_informar_path =
            data_dir.join(format!("ventas_sin_informar_{}.json", self.estacion_id));

        let ventas_vacias: HashMap<usize, HashMap<usize, Vec<Venta>>> = HashMap::new();
        self.save_json_nested_hashmap(&ventas_sin_informar_path, &ventas_vacias)
    }

    /// Carga un HashMap anidado desde un archivo JSON.
    pub fn load_json_nested_hashmap<T: for<'de> serde::Deserialize<'de>>(
        &self,
        path: &Path,
    ) -> Result<HashMap<usize, HashMap<usize, Vec<T>>>, Error> {
        match File::open(path) {
            Ok(f) => match serde_json::from_reader(f) {
                Ok(v) => Ok(v),
                Err(e) => {
                    eprintln!("No se pudo parsear {}: {}", path.display(), e);
                    Ok(HashMap::new())
                }
            },
            Err(e) => {
                eprintln!("No se pudo abrir {}: {}", path.display(), e);
                Ok(HashMap::new())
            }
        }
    }

    /// Guarda un HashMap anidado en un archivo JSON.
    pub fn save_json_nested_hashmap<T: serde::Serialize>(
        &self,
        path: &Path,
        data: &HashMap<usize, HashMap<usize, Vec<T>>>,
    ) -> Result<(), Error> {
        match File::create(path) {
            Ok(mut file) => match serde_json::to_writer_pretty(&mut file, data) {
                Ok(_) => match file.flush() {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let error_msg = format!(
                            "No se pudo hacer flush del archivo {}: {}",
                            path.display(),
                            e
                        );
                        eprintln!("{}", error_msg);
                        Err(Error::ErrorString(error_msg))
                    }
                },
                Err(e) => {
                    let error_msg =
                        format!("No se pudo serializar datos en {}: {}", path.display(), e);
                    eprintln!("{}", error_msg);
                    Err(Error::ErrorString(error_msg))
                }
            },
            Err(e) => {
                let error_msg = format!("No se pudo crear el archivo {}: {}", path.display(), e);
                eprintln!("{}", error_msg);
                Err(Error::ErrorString(error_msg))
            }
        }
    }
}
