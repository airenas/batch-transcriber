-include Makefile.options
log?=INFO
###############################################################################
run/worker:
	RUST_LOG=$(log) cargo run --bin worker -- --base-dir=./test -w=2 \
		--asr-url=https://atpazinimas.intelektika.lt/ausis --asr-auth-key=
.PHONY: run/worker
###############################################################################
run/sound-keeper:
	RUST_LOG=$(log) cargo run --bin sound-keeper -- --base-dir=./test
.PHONY: run/sound-keeper
###############################################################################
build/local: 
	cargo build --release
.PHONY: build/local
###############################################################################
test/unit:
	RUST_LOG=DEBUG cargo test --no-fail-fast
.PHONY: test/unit		
test/lint:
	@cargo clippy -V
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: test/lint	
###############################################################################
diesel/migrate:
	diesel migration run
diesel/generate: diesel/migrate
	@echo "File schema.rs created. move it to model/schema.rs!!"
diesel/redo:
	diesel migration redo
diesel/revert:
	diesel migration revert	
diesel/check-last:
	diesel migration redo
diesel/new/migration:
	diesel migration generate $(name)


.EXPORT_ALL_VARIABLES:

