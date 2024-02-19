os := os_family()

build-cloud NAME:
  cargo build --package cloud-{{NAME}} --release

make-ln NAME LIB:
  ln -s {{justfile_directory()}}/target/release/libcloud_{{LIB}}.so {{justfile_directory()}}/clouds/{{NAME}}/libcloud_{{LIB}}.so || echo ""

setup-linux: \
  (build-cloud "http-router") \
  (build-cloud "views") \
  (make-ln "http-router" "http_router") \
  (make-ln "views" "views")

setup:
  if [[ {{os}} = unix ]]; then \
    just setup-linux; \
  else \
    echo "No setup "; \
  fi

run +COMMAND="dev": (setup)
  cd example_server && cargo run --package densky -- {{COMMAND}}
