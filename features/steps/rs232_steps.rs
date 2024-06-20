use crate::world::MyWorld;
use cucumber::{given, then, /*when*/};

use udev::{Enumerator, /*Device*/};
use std::time::Duration;
use serialport::SerialPort;


fn find_serial_port() -> Option<String> {
    let mut enumerator = Enumerator::new().unwrap();
    enumerator.match_subsystem("tty").unwrap();
    for device in enumerator.scan_devices().unwrap() {
        if let Ok(Some(parent)) = device.parent_with_subsystem_devtype("usb", "usb_device") {
            if let Some(id_vendor) = parent.attribute_value("idVendor") {
                if let Some(id_product) = parent.attribute_value("idProduct") {
                    if id_vendor.to_str() == Some("2e8a") && id_product.to_str() == Some("0005") {
                        return Some(device.devnode().unwrap().to_str().unwrap().to_string());
                    }
                }
            }
        }
    }
    None
}

fn open_serial(serial_port_name: &Option<String>) -> Box<dyn SerialPort>{
    if let Some(s) = serial_port_name {
        let port = serialport::new(s, 9600)
            .timeout(Duration::from_millis(1000))
            .open()
            .expect("Failed to open port");
        port
    }
    else {
        panic!("serial port name is not set");
    }
}

fn send(message: &str, port: &mut Box<dyn SerialPort>) {
    port.write(message.as_bytes()).expect("Failed to write to serial port");
}

fn read(port: &mut Box<dyn SerialPort>) -> Vec<u8>{
    let mut serial_buf: Vec<u8> = vec![0; 32];
    port.read(serial_buf.as_mut_slice()).expect("Found no data!");
    serial_buf
}

#[given("the connection to the device via USB")]
fn usb_connection(world: &mut MyWorld) {
    world.serial_port_name = find_serial_port();
    assert!(world.serial_port_name.is_some());
}

#[then("a serial connection can be established")]
fn serial_connection(world: &mut MyWorld) {
    let mut port = open_serial(&world.serial_port_name);
    send("ping", &mut port);
    let answer = String::from_utf8(read(&mut port)).expect("Failed to read from serial port");
    assert_eq!(answer, "pong");
}
