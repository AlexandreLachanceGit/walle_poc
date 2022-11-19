build:
	cd main && cross build --release --target x86_64-unknown-linux-musl
	cp main/target/x86_64-unknown-linux-musl/release/walle_poc main/target/bootstrap
	cd main/terraform && terraform plan -out tfplan.out

deploy:
	cd main/terraform && terraform apply "tfplan.out"
	cd register && cargo run --release ../commands.json 
