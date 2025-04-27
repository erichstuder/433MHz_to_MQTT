use crate::MyWorld;
use crate::rs232::{find_serial_port, clear_input_buffer, open_serial, send, read};
use cucumber::{given, then};

#[given("the connection to the device via USB")]
fn usb_connection(world: &mut MyWorld) {
    world.serial_port_name = find_serial_port();
    assert!(world.serial_port_name.is_some());
}

#[then("a serial connection can be established")]
fn serial_connection(world: &mut MyWorld) {
    let mut port = open_serial(&world.serial_port_name);
    clear_input_buffer(&mut port);
    send("ping\n", &mut port);
    let answer = String::from_utf8(read(&mut port)).expect("Failed to read from serial port");
    let answer = answer.trim_end_matches('\0');
    assert_eq!(answer, "pong");
}
