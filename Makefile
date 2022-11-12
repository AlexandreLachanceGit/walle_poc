build:
	cross build --release --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/release/walle_poc ./bootstrap
	terraform plan -out tfplan.out

deploy:
	terraform apply "tfplan.out"
	cd register_test && node index.js
