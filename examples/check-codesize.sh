#!/usr/bin/env -S bash -e

release_dir=target/wasm32-unknown-unknown/release

files="\
	bobcat_sdk_examples_001.wasm \
	bobcat-sdk-examples-002.wasm \
	bobcat-sdk-examples-003.wasm"

in_err=0

err() {
	>&2 echo "$1 size regression (expected $2, is $3)"
	in_err=1
}

check_size() {
	s="$(du -b $1 | cut -f1)"
	if [ $s -gt $2 ]; then err $1 $2 $s; fi
}

for n in $files; do
	f="$release_dir/$n"
	size="$(du -b $f)"
	case $n in
		bobcat_sdk_examples_001.wasm) check_size $f 5592 ;;
		bobcat-sdk-examples-002.wasm) check_size $f 4337 ;;
		bobcat-sdk-examples-003.wasm) check_size $f 4475 ;;
	esac
done

exit $in_err
