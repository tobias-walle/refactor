run *ARGS:
   RUST_BACKTRACE=1 cargo run {{ARGS}}

fix:
  cargo clippy --fix --allow-staged

build:
  cargo build --release

link:
  cargo install --path .
