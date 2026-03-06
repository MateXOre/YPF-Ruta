use std::fmt;

/// Errores personalizados para la aplicación
#[derive(Debug)]
pub enum Error {
    ErrorString(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ErrorString(e) => {
                write!(f, "{}", e)
            }
        }
    }
}
