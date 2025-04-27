use crate::MyWorld;
use crate::rs232::{find_and_open_serial, clear_input_buffer, send, read};
use cucumber::{given, when, then};

#[given("the communication to the device over RS232")]
fn usb_connection(world: &mut MyWorld) {
    let port = find_and_open_serial();
    world.port = Some(port.expect("Serial port find and open failed"));
}

#[when(regex = r"^the command is sent: 'store (\S+) (\S+)\\\\n'$")]
fn send_store_command(world: &mut MyWorld, parameter_name: String, value_example: String) {
    let command = format!("store {} {}\n", parameter_name, value_example);
    if let Some(ref mut port) = world.port {
        send(&command, port);
    } else {
        panic!("Serial port not initialized");
    }
}

// #[when("the device is power cycled")]
// fn power_cycle_device(_world: &mut MyWorld) {
//     //TODO: implement real power cycle and then reconnect
//     //world.port = find_and_open_serial();
// }


#[when(regex = r"^the command is sent: 'read (\S+)\\\\n'$")]
fn send_read_command(world: &mut MyWorld, parameter_name: String) {
    let command = format!("read {}\n", parameter_name);
    if let Some(ref mut port) = world.port {
        clear_input_buffer(port);
        send(&command, port);
    } else {
        panic!("Serial port not initialized");
    }
}

#[then(regex = r"^the answer is: '(\S+)\\\\n'$")]
fn verify_answer(world: &mut MyWorld, expected_value: String) {
    if let Some(ref mut port) = world.port {
        let answer = String::from_utf8(read(port)).expect("Failed to read from serial port");
        let answer = answer.trim_end_matches('\0').trim();
        assert_eq!(answer, expected_value);
    } else {
        panic!("Serial port not initialized");
    }
}
