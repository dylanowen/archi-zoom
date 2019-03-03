SHELL:=/bin/bash

.DEFAULT_GOAL := default

format:
	cargo fmt

build: format
	wasm-pack build --target no-modules

dot:
	+$(MAKE) -C web/diagrams

default: dot build

clean:
	cargo clean