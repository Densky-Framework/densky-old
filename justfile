
run +COMMAND="dev":
  cargo build --package cloud-http-router --release
  cd example_server && cargo run --package densky -- {{COMMAND}}
