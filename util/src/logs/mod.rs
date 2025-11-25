pub mod logger;
/*
A continuación se definen macros para enviar logs a través del logger.

Estas macros encapsulan la logica de tener que crear un mensaje con format! y luego
enviarlo al logger a través del canal de comunicación.

Ya incluyen el prefijo correspondiente para cada tipo de log (INFO, ERROR, WARN, DEBUG).

Se pueden usar de la siguiente manera:
log_info!(logger, "Este es un mensaje informativo con un valor: {}", valor);

El log debug lo pueden usar mientras crean nuevas features para después poder eliminarlos
más fácil buscando el 'log_debug!' en el código.
*/

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let msg = format!("[INFO] {}", format!($fmt $(, $arg)*));
        if let Err(e) = $logger.send(msg.into_bytes()) {
            eprintln!("Error enviando log: {}", e);
        }
    }};
}
pub use log_info;

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let msg = format!("[ERROR] {}", format!($fmt $(, $arg)*));
        if let Err(e) = $logger.send(msg.into_bytes()) {
            eprintln!("Error enviando log: {}", e);
        }
    }};
}
pub use log_error;

#[macro_export]
macro_rules! log_warning {
    ($logger:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let msg = format!("[WARN] {}", format!($fmt $(, $arg)*));
        if let Err(e) = $logger.send(msg.into_bytes()) {
            eprintln!("Error enviando log: {}", e);
        }
    }};
}
pub use log_warning;

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let msg = format!("[DEBUG] {}", format!($fmt $(, $arg)*));
        if let Err(e) = $logger.send(msg.into_bytes()) {
            eprintln!("Error enviando log: {}", e);
        }
    }};
}
pub use log_debug;
