use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::mpsc::{Receiver, Sender, channel};

use chrono::{FixedOffset, Utc};

pub struct Logger {
    tx_channel: Sender<Vec<u8>>,
}

impl Logger {
    pub fn new(path: &str) -> Option<Logger> {
        let file = match OpenOptions::new().create(true).append(true).open(path) {
            Ok(f) => f,
            Err(err) => {
                println!("ERROR AL ABRIR EL ARCHIVO: {}", err);
                return None;
            }
        };

        let writer = BufWriter::new(file);

        let (tx, rx) = channel();

        Logger::save_logs(rx, writer);

        Some(Logger { tx_channel: tx })
    }

    pub fn get_log_channel(self) -> Sender<Vec<u8>> {
        self.tx_channel.clone()
    }

    pub fn log_message(channel: &Sender<Vec<u8>>, msg: &str) {
        if let Err(err) = channel.send(msg.as_bytes().to_vec()) {
            println!("ERROR AL ENVIAR EL MENSAJE AL LOG: {}", err);
        }
    }

    pub fn info(channel: &Sender<Vec<u8>>, args: std::fmt::Arguments) {
        let log = format!("[INFO] {}", args);
        Logger::log_message(channel, &log);
    }

    pub fn error(channel: &Sender<Vec<u8>>, args: std::fmt::Arguments) {
        let log = format!("[ERROR] {}", args);
        Logger::log_message(channel, &log);
    }

    pub fn warning(channel: &Sender<Vec<u8>>, args: std::fmt::Arguments) {
        let log = format!("[WARNING] {}", args);
        Logger::log_message(channel, &log);
    }

    pub fn debug(channel: &Sender<Vec<u8>>, args: std::fmt::Arguments) {
        let log = format!("[DEBUG] {}", args);
        Logger::log_message(channel, &log);
    }

    fn format_log(msg: &mut Vec<u8>) -> Option<Vec<u8>> {
        let mut log: Vec<u8> = Vec::new();
        let timezone = FixedOffset::west_opt(3 * 3600).unwrap();
        let time = Utc::now()
            .with_timezone(&timezone)
            .format("%d/%m/%Y - %H:%M:%S");

        log.extend_from_slice(time.to_string().as_bytes());
        log.push(b' ');
        log.append(msg);
        log.push(b'\n');
        Some(log)
    }

    fn save_logs(rx_channel: Receiver<Vec<u8>>, file: BufWriter<File>) {
        let mut writer = file;
        std::thread::spawn(move || {
            while let Ok(mut msg) = rx_channel.recv() {
                let log = match Logger::format_log(&mut msg) {
                    Some(value) => value,
                    None => return,
                };

                if let Err(err) = writer.write_all(&log) {
                    println!("ERROR AL LOGGEAR EN EL ARCHIVO: {}", err);
                };
                if let Err(err) = writer.flush() {
                    println!("ERROR AL FLUSH: {}", err);
                }
            }
        });
    }
}