cargo build
cargo test --no-default-features --features target-test
cargo test --no-default-features --features host-test --target x86_64-unknown-linux-gnu
