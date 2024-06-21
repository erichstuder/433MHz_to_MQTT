mod world;
mod rs232;
mod rs232_steps;
mod persistency_steps;

use cucumber::World;
use world::MyWorld;

// This runs before everything else, so you can setup things here.
#[tokio::main]
async fn main() {
    // You may choose any executor you like (`tokio`, `async-std`, etc.).
    // You may even have an `async` main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    MyWorld::cucumber()
    .fail_on_skipped()
    .run_and_exit("..")
    .await;
}
