use udev::Enumerator;
use std::time::Duration;
use serialport::SerialPort;

pub fn find_serial_port() -> Option<String> {
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

pub fn open_serial(serial_port_name: &Option<String>) -> Box<dyn SerialPort>{
    if let Some(s) = serial_port_name {
        let port = serialport::new(s, 9600)
            .timeout(Duration::from_secs(2))
            .open()
            .expect("Failed to open port");
        port
    }
    else {
        panic!("serial port name is not set");
    }
}

pub fn find_and_open_serial() -> Option<Box<dyn SerialPort>> {
    let serial_port_name = find_serial_port();
    assert!(serial_port_name.is_some());
    Some(open_serial(&serial_port_name))
}

pub fn clear_input_buffer(port: &mut Box<dyn SerialPort>) {
    let mut buffer = [0; 1024]; // Adjust the buffer size if necessary
    while let Ok(bytes_read) = port.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
    }
}

pub fn send(message: &str, port: &mut Box<dyn SerialPort>) {
    port.write(message.as_bytes()).expect("Failed to write to serial port");
    port.flush().expect("Failed to flush serial port");
}

pub fn read(port: &mut Box<dyn SerialPort>) -> Vec<u8>{
    let mut serial_buf: Vec<u8> = vec![0; 32];
    port.read(serial_buf.as_mut_slice()).expect("Found no data!");
    serial_buf
}
