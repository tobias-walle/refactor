run *ARGS:
  cargo run {{ARGS}}

fix:
  cargo clippy --fix --allow-staged

build:
  cargo build --release

link:
  cargo install --path .
