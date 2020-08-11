#!/bin/sh

cd rust

cargo build && \
	~/.cargo/bin/wasm-pack build && \
	npm run build && \
	cd ../ && \
	npm run start
