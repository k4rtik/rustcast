all:
	cargo build --release && mv ./target/release/rustcast_* .

clean:
	cargo clean
	rm -rf Cargo.lock target rustcast_*
