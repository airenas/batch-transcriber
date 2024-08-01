-include Makefile.options
log?=INFO
###############################################################################
run/worker:
	RUST_LOG=$(log) cargo run --bin worker -- --base-dir=./test -w=2 \
		--asr-url=https://atpazinimas.intelektika.lt/ausis --asr-auth-key=
.PHONY: run/worker
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
generate/db:
	diesel migration run

.EXPORT_ALL_VARIABLES:

