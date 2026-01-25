
.DELETE_ON_ERROR:

.PHONY: build clean

CARGO_EXTRA_FEATURES := \
	$(if ${BOZO_INLINE_MATHS},--features inline-maths)

build: bozo.wasm bootstrap.zip

bozo.wasm: $(shell find Cargo.* src -type f)
	@cargo build \
		--release \
		--target wasm32-unknown-unknown \
		${CARGO_EXTRA_FEATURES}
	@./wasm-post.sh \
		target/wasm32-unknown-unknown/release/bobcat-examples-004.wasm \
		bozo.wasm
	@./check-codesize.sh bozo.wasm

bozo: $(shell find Cargo.* src -type f)
	@cargo build \
		--release \
		--target riscv32im-unknown-none-elf \
		${CARGO_EXTRA_FEATURES}
	@cp target/riscv32im-unknown-none-elf/release/bobcat-examples-004 bozo
	@./check-codesize.sh bobcat-examples-004

bozo.out: $(shell find -name 'go.*' -or -name '*.go' -type f)
	@go build -o bozo.out

bootstrap: bozo.out
	@cp bozo.out bootstrap

bootstrap.zip: bootstrap
	@zip bootstrap.zip bootstrap

clean:
	@rm -rf bootstrap.zip bootstrap bozo.wasm target bozo.out
