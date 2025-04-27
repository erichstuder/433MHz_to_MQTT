mod rs232;
mod step_definitions;

use cucumber::World;
use serialport::SerialPort;

#[derive(Debug, Default, World)]
pub struct MyWorld {
    pub serial_port_name: Option<String>,
    pub port: Option<Box<dyn SerialPort>>,
}

#[tokio::main]
async fn main() {
    // Note:
    // Only one scenario is run at a time as at the moment all Scenarios need the USB connection to the device.
    // So concurrent execution is not possible, wich is achieve with max_concurrent_scenarios(1).
    // If there are ever Scenarios that can be run concurrently see here: https://github.com/cucumber-rs/cucumber/issues/367
    MyWorld::cucumber().max_concurrent_scenarios(1).run("..").await;
}
