extern crate cucumber;
extern crate futures;

use cucumber::{given, when, then, World};

// These `Cat` definitions would normally be inside your project's code,
// not test code, but we create them here for the show case.
#[derive(Debug, Default)]
struct Cat {
    pub hungry: bool,
}

impl Cat {
    fn feed(&mut self) {
        self.hungry = false;
    }
}

// `World` is your shared, likely mutable state.
// Cucumber constructs it via `Default::default()` for each scenario.
#[derive(Debug, Default, World)]
pub struct AnimalWorld {
    cat: Cat,
}

// Steps are defined with `given`, `when` and `then` attributes.
#[given("a hungry cat")]
fn hungry(world: &mut AnimalWorld) {
    world.cat.hungry = true;
}

#[when("I feed the cat")]
fn feed(world: &mut AnimalWorld) {
    world.cat.feed();
}

#[then("the cat is not hungry")]
fn not_hungry(world: &mut AnimalWorld) {
    assert!(!world.cat.hungry);
}

// This runs before everything else, so you can setup things here.
fn main() {
    // You may choose any executor you like (`tokio`, `async-std`, etc.).
    // You may even have an `async` main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    futures::executor::block_on(AnimalWorld::run("."));
}
