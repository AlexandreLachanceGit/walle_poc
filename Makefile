build:
	cross build --release --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/release/walle_poc ./bootstrap
	