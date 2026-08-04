#!/bin/sh -eu

echo $1 | python3 -c '
import hashlib
import sys

seed = sys.stdin.buffer.read()
size = 1 + int.from_bytes(hashlib.sha256(seed).digest()[:4], "big") % (70 * 1024)
sys.stdout.buffer.write(hashlib.shake_256(seed).digest(size))' \
	| xz \
	| xxd -ps -c0
