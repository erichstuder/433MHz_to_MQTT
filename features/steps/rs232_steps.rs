use crate::world::MyWorld;
use cucumber::{given, /*then, when*/};

//use serialport::prelude::*;
//use std::ffi::OsStr;
use udev::{Enumerator, /*Device*/};


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

fn send(_message: &str) {
    let serial_port_name = find_serial_port();
    if let Some(s) = serial_port_name {
        //printout serial port name
        println!("Serial port name: {}", s);
    }
    else {
        println!("Serial port not found");
    }
    println!("dooooooooooooooooooooooooooooooooooooooone");

    // if let Some(s) = serial_port_name {
    //     let mut port = serialport::open(&s).expect("Failed to open serial port");

    //     let mut settings: SerialPortSettings = Default::default();
    //     settings.timeout = Duration::from_millis(100);
    //     settings.baud_rate = 9600;
    //     settings.data_bits = DataBits::Eight;
    //     settings.parity = Parity::None;
    //     settings.stop_bits = StopBits::One;
    //     settings.flow_control = FlowControl::None;
    //     port.set_all(&settings).expect("Failed to set port settings");

    //     port.write(message.as_bytes()).expect("Failed to write to serial port");
    // }
}

#[given("the connection to the device via USB")]
fn usb_connection(world: &mut MyWorld) {
    let _ = world; //prevent unused warning for the moment
    send("dummy");
}
