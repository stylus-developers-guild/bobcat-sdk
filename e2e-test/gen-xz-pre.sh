#!/bin/sh -eu

# Like gen-xz-cd.sh except we don't do the xz compression step.

echo $1 | python3 -c '
import hashlib
import sys

seed = sys.stdin.buffer.read()
size = 1 + int.from_bytes(hashlib.sha256(seed).digest()[:4], "big") % (70 * 1024)
sys.stdout.buffer.write(hashlib.shake_256(seed).digest(size))' \
	| xxd -ps -c0
