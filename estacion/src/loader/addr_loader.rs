use crate::errors::error::Error;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::net::IpAddr;
use std::net::SocketAddr;

const ARCHIVO_NOMBRE: &str = "estaciones.csv";

pub struct AddrLoader {}

impl AddrLoader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_all(&self) -> Result<Vec<SocketAddr>, Error> {
        let file = match fs::File::open(ARCHIVO_NOMBRE) {
            Ok(file) => file,
            Err(e) => return Err(Error::ErrorString(e.to_string())),
        };
        let reader = BufReader::new(file);
        let mut addresses = Vec::new();
        for line in reader.lines().skip(1) {
            let line = match line {
                Ok(line) => line,
                Err(e) => return Err(Error::ErrorString(e.to_string())),
            };
            let addr = self.parse_line(&line)?;
            addresses.push(addr);
        }
        Ok(addresses)
    }

    pub fn load_line(&self, line_number: usize) -> Result<SocketAddr, Error> {
        let file = match fs::File::open(ARCHIVO_NOMBRE) {
            Ok(file) => file,
            Err(e) => return Err(Error::ErrorString(e.to_string())),
        };
        let reader = BufReader::new(file);
        let line = match reader.lines().skip(1).nth(line_number) {
            Some(line) => line,
            None => return Err(Error::ErrorString("Linea invalida".to_string())),
        };
        let line = match line {
            Ok(line) => line,
            Err(e) => return Err(Error::ErrorString(e.to_string())),
        };
        let addr = self.parse_line(&line)?;
        Ok(addr)
    }

    fn parse_line(&self, line: &str) -> Result<SocketAddr, Error> {
        let splitted = line.trim().split(",").collect::<Vec<&str>>();
        if splitted.len() != 2 {
            return Err(Error::ErrorString("Linea invalida".to_string()));
        }
        let ip_str = splitted[0];
        let port_str = splitted[1];
        let ip = self.get_ip_from_string(ip_str)?;
        let port = self.get_port_from_string(port_str)?;
        let addr = self.get_addr(ip, port)?;
        Ok(addr)
    }

    fn get_ip_from_string(&self, ip_str: &str) -> Result<IpAddr, Error> {
        let ip = match ip_str.parse::<IpAddr>() {
            Ok(ip) => ip,
            Err(e) => return Err(Error::ErrorString(e.to_string())),
        };
        Ok(ip)
    }

    fn get_port_from_string(&self, port_str: &str) -> Result<u16, Error> {
        let port = match port_str.parse::<u16>() {
            Ok(port) => port,
            Err(e) => return Err(Error::ErrorString(e.to_string())),
        };
        Ok(port)
    }

    fn get_addr(&self, ip: IpAddr, port: u16) -> Result<SocketAddr, Error> {
        let addr = match format!("{}:{}", ip, port).parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => return Err(Error::ErrorString(e.to_string())),
        };
        Ok(addr)
    }
}
