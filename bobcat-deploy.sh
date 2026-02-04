#!/bin/sh -e

# Note on the arguments: generally speaking, it's better to use
# environment variables for private information. Environment variables
# are shielded from the process table, which is usually not private, so
# you couldn't use these tools safely on a time shared system (aka, not
# your own). Unfortunately, Foundry doesn't care about this, and
# no-one uses Seth, so we externalise this to you.

endpoint="$1"
private_key="$2"
wasm_file="$3"

usage() {
	>&2 echo $(basename $0) "<endpoint> <private key> <wasm file>"
	exit 2
}

[ -z "$endpoint" ] && usage
[ -z "$private_key" ] && usage
[ -z "$wasm_file" ] && usage

if [ -z "$BC_CONTRACT" ]; then
	compressed="eff00000$(brotli -cq 11 $wasm_file | xxd -p -c0)"

	len="$(printf '%08x' $(echo $(echo -n "$compressed" | wc -c) ' / 2' | bc))"

	with_preamble="63${len}80600c5f395ff3$compressed"

	BC_CONTRACT="$(\
		cast send \
			--json \
			--rpc-url "$endpoint" \
			--private-key "$private_key" \
			--create "0x$with_preamble" \
				| jq -r .contractAddress)"
fi

echo $BC_CONTRACT

arb_wasm=0x0000000000000000000000000000000000000071

random_sender="0x$(dd count=20 bs=1 status=none if=/dev/urandom | xxd -p)"

max_eth=115792089237316195423570985008687907853269984665640564039457584007913129639935

if [ -z "$BC_ACTIVATION" ]; then
	export BC_ACTIVATION="$(\
		cast call \
			--json \
			--rpc-url "$endpoint" \
			--override-balance "$random_sender:$max_eth" \
			--from "$random_sender" \
			--value "$max_eth" \
			$arb_wasm \
			"activateProgram(address)(uint16,uint256)" \
			$BC_CONTRACT \
				| jq -r '.[1]')"
fi

cast send \
	--json \
	--rpc-url "$endpoint" \
	--private-key "$private_key" \
	--value "$BC_ACTIVATION" \
	"$arb_wasm" \
	"activateProgram(address)" \
	"$BC_CONTRACT" >/dev/null
