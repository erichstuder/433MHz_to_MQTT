use cucumber::World;

#[derive(Debug, Default, World)]
pub struct MyWorld {
    pub serial_port_name: Option<String>,
}
