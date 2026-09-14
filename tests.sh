#!/bin/sh -e

features=std,proptest,alloy-enabled,alloc

cargo test --no-default-features --features $features -- --nocapture $@

cd examples

make

cd ../e2e-test

./tests.sh
