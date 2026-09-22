
.DELETE_ON_ERROR:

RELEASE := target/wasm32-unknown-unknown/release

BUILT := ${RELEASE}/beacon.d

PROJECTS := \
	chainlink-vrf-test \
	transient-exchange \
	console-log-txt \
	counter \
	mul-div \
	beacon \
	panic-revert \
	eip1967-proxy \
	reverts-bubbling-up \
	proxy-predicts-create2 \
	precompiles \
	slot-map \
	safe-erc20 \
	tracing-counter \
	panic-trace \
	shadow \
	features \
	zeroise-check \
	slot-key \
	derive-trace

all: wasm

.PHONY: wasm clean

${BUILT}: $(shell find ${PROJECTS} Cargo.toml -type f)
	@cargo build --target wasm32-unknown-unknown --release

wasm: \
	chainlink-vrf-test.wasm \
	transient-exchange.wasm \
	console-log-txt.wasm \
	counter.wasm \
	mul-div.wasm \
	beacon.wasm \
	panic-revert.wasm \
	eip1967-proxy.wasm \
	reverts-bubbling-up.wasm \
	proxy-predicts-create2.wasm \
	precompiles.wasm \
	slot-map.wasm \
	safe-erc20.wasm \
	tracing-counter.wasm \
	panic-trace.wasm \
	shadow.wasm \
	features.wasm \
	zeroise-check.wasm \
	slot-key.wasm \
	derive-trace.wasm

chainlink-vrf-test.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/chainlink-vrf-test.wasm chainlink-vrf-test.wasm

transient-exchange.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/transient-exchange.wasm transient-exchange.wasm

console-log-txt.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/console-log-txt.wasm console-log-txt.wasm

counter.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/libcounter.wasm counter.wasm

mul-div.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/mul-div.wasm mul-div.wasm

beacon.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/beacon.wasm beacon.wasm

panic-revert.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/panic-revert.wasm panic-revert.wasm

eip1967-proxy.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/eip1967-proxy.wasm eip1967-proxy.wasm

reverts-bubbling-up.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/reverts-bubbling-up.wasm reverts-bubbling-up.wasm

proxy-predicts-create2.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/proxy-predicts-create2.wasm proxy-predicts-create2.wasm

precompiles.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/precompiles.wasm precompiles.wasm

slot-map.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/slot-map.wasm slot-map.wasm

safe-erc20.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/safe-erc20.wasm safe-erc20.wasm

tracing-counter.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/tracing-counter.wasm tracing-counter.wasm

panic-trace.wasm: $(shell find panic-trace/src -type f)
	@cd panic-trace && cargo build --release --target wasm32-unknown-unknown
	@./wasm-post.sh panic-trace/target/wasm32-unknown-unknown/release/panic-trace.wasm panic-trace.wasm

shadow.wasm: $(shell find shadow/src -type f)
	@cd shadow && cargo build --release --target wasm32-unknown-unknown
	@./wasm-post.sh shadow/target/wasm32-unknown-unknown/release/shadow.wasm shadow.wasm

features.wasm: ${BUILT}
	@# We don't post process features since wat2wasm hangs due to some kind of bug:
	@cp ${RELEASE}/features.wasm features.wasm

zeroise-check.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/zeroise-check.wasm zeroise-check.wasm

slot-key.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/slot-key.wasm slot-key.wasm

derive-trace.wasm: ${BUILT}
	@./wasm-post.sh ${RELEASE}/derive-trace.wasm derive-trace.wasm

clean:
	@rm -rf \
		target \
		chainlink-vrf-test.wasm \
		transient-exchange.wasm \
		console-log-txt.wasm \
		counter.wasm \
		mul-div.wasm \
		beacon.wasm \
		panic-revert.wasm \
		eip1967-proxy.wasm \
		reverts-bubbling-up.wasm \
		proxy-predicts-create2.wasm \
		precompiles.wasm \
		slot-map.wasm \
		safe-erc20.wasm \
		tracing-counter.wasm \
		panic-trace.wasm \
		shadow.wasm \
		features.wasm \
		zeroise-check.wasm \
		slot-key.wasm \
		derive-trace.wasm
