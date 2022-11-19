build:
	cross build --release --target x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/release/walle_poc target/bootstrap
	cd terraform && terraform plan -out tfplan.out

deploy:
	cd terraform && terraform apply "tfplan.out"
	cd register_test && node index.js
