
.PHONY: clean build

OBJ_DIR := target/wasm32-unknown-unknown/release

OBJ_COUNTER := ${OBJ_DIR}/bobcat_sdk_examples_001.wasm
OBJ_MUL_DIV := ${OBJ_DIR}/bobcat-sdk-examples-002.wasm
OBJ_CAMELOT := ${OBJ_DIR}/bobcat-sdk-examples-003.wasm

build: 001-counter.wasm 002-mul_div.wasm 003-camelot.wasm

${OBJ_COUNTER}: $(shell find Cargo.* 001-counter 002-mul_div 003-camelot-swapping -type f -name Cargo.toml -or -name '*.rs')
	@cargo build --target wasm32-unknown-unknown --release

001-counter.wasm: ${OBJ_COUNTER}
	@./wasm-post.sh ${OBJ_COUNTER} 001-counter.wasm

002-mul_div.wasm: ${OBJ_MUL_DIV}
	@./wasm-post.sh ${OBJ_MUL_DIV} 002-mul_div.wasm

003-camelot.wasm: ${OBJ_CAMELOT}
	@./wasm-post.sh ${OBJ_CAMELOT} 003-camelot.wasm

clean:
	@cargo clean
	@rm -f 001-counter.wasm 002-mul_div.wasm 003-camelot.wasm

