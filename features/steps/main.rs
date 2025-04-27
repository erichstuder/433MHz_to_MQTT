mod world;
mod rs232;
mod rs232_steps;
mod persistency_steps;

use cucumber::World;
use world::MyWorld;

#[tokio::main]
async fn main() {
    // Note:
    // Only one scenario is run at a time as at the moment all Scenarios need the USB connection to the device.
    // So concurrent execution is not possible, wich is achieve with max_concurrent_scenarios(1).
    // If there are ever Scenarios that can be run concurrently see here: https://github.com/cucumber-rs/cucumber/issues/367
    MyWorld::cucumber().max_concurrent_scenarios(1).run("..").await;
}
