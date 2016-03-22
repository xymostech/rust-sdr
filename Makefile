out.tga: $(wildcard src/*.rs) Cargo.toml
	cargo run
