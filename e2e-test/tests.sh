#!/bin/sh -e

make -B

# export FOUNDRY_FUZZ_RUNS=10000

arbos-forge test --gas-snapshot-check true --ffi --stylus-debug $@
