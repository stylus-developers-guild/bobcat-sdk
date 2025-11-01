#!/bin/sh

url=${BOZO_ENDPOINT:-https://rpc.superposition.so}

if [ -z "$BOZO_PRIVATE_KEY" ]; then
	>&2 echo "BOZO_PRIVATE_KEY unset"
	exit 2
fi

cargo stylus deploy \
	--wasm-file bozo.wasm \
	--private-key "$BOZO_PRIVATE_KEY" \
	--endpoint "$url" \
	--no-verify \
	        | sed -nr 's/.*deployed code at address: +.*(0x.{40}).*$/\1/p'
