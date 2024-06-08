use crate::world::MyWorld;
use cucumber::{given, /*then, when*/};


#[given("the connection to the device via USB")]
fn usb_connection(world: &mut MyWorld) {
    let _ = world;
}
