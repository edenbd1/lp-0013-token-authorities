.PHONY: build test demo fmt clippy clean

CARGO ?= cargo

build:
	$(CARGO) build --release --workspace

test:
	$(CARGO) test --workspace --release

demo:
	./scripts/demo.sh

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace -- -D warnings

clean:
	$(CARGO) clean
