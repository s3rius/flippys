.PHONY: b
b:
	cargo build --release

.PHONY: u
u:
	storage send ./target/thumbv7em-none-eabihf/release/btcon_client.fap /ext/apps/Examples/btcon_client.fap
