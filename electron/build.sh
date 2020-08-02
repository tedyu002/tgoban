#!/bin/sh

cd rust

cargo build && \
	~/.cargo/bin/wasm-pack build && \
	cd www && \
	npm run build &&
	cd ../.. && \
	npm run start
