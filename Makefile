all:
	cargo build --release && mv ./target/release/snowcast_* .

clean:
	cargo clean
	rm -rf Cargo.lock target snowcast_*
