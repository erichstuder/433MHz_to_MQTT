use cucumber::World;
use serialport::SerialPort;

#[derive(Debug, Default, World)]
pub struct MyWorld {
    pub serial_port_name: Option<String>,
    pub port: Option<Box<dyn SerialPort>>,
}
