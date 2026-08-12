#!/usr/bin/env bash
set -euo pipefail

RELEASE_VERSION="${RELEASE_VERSION:-$(python3 - <<'PY'
import tomllib

with open("bobcat-sdk/Cargo.toml", "rb") as f:
    print(tomllib.load(f)["package"]["version"])
PY
)}"
export RELEASE_VERSION

metadata="$(mktemp)"
packages="$(mktemp)"
trap 'rm -f "$metadata" "$packages"' EXIT

cargo metadata --no-deps --format-version 1 > "$metadata"

python3 - "$metadata" "$RELEASE_VERSION" > "$packages" <<'PY'
import json
import sys

metadata_path, release_version = sys.argv[1:]
with open(metadata_path, encoding="utf-8") as f:
    metadata = json.load(f)

workspace_ids = set(metadata["workspace_members"])
workspace = {
    package["name"]: package
    for package in metadata["packages"]
    if package["id"] in workspace_ids and package["publish"] != []
}

wrong_versions = sorted(
    f"{name}={package['version']}"
    for name, package in workspace.items()
    if package["version"] != release_version
)
if wrong_versions:
    raise SystemExit(
        "all publishable workspace crates must use release version "
        f"{release_version}; mismatches: {', '.join(wrong_versions)}"
    )

dependencies = {
    name: {
        dependency["name"]
        for dependency in package["dependencies"]
        if dependency.get("path") is not None and dependency["name"] in workspace
    }
    for name, package in workspace.items()
}

remaining = set(workspace)
ordered = []
while remaining:
    ready = sorted(name for name in remaining if not (dependencies[name] & remaining))
    if not ready:
        raise SystemExit(
            "cycle in publishable workspace dependencies: " + ", ".join(sorted(remaining))
        )
    ordered.extend(ready)
    remaining.difference_update(ready)

print("\n".join(ordered))
PY

crate_exists() {
    local crate="$1"
    local status

    status="$(curl \
        --silent \
        --show-error \
        --location \
        --retry 3 \
        --user-agent "bobcat-sdk-release (https://github.com/stylus-developers-guild/bobcat-sdk)" \
        --output /dev/null \
        --write-out '%{http_code}' \
        "https://crates.io/api/v1/crates/${crate}/${RELEASE_VERSION}")"

    case "$status" in
        200) return 0 ;;
        404) return 1 ;;
        *)
            echo "unexpected crates.io response for ${crate} ${RELEASE_VERSION}: HTTP ${status}" >&2
            return 2
            ;;
    esac
}

while IFS= read -r crate; do
    [ -n "$crate" ] || continue

    if crate_exists "$crate"; then
        echo "${crate} ${RELEASE_VERSION} is already published; skipping"
        continue
    fi

    echo "Publishing ${crate} ${RELEASE_VERSION}"
    published=false
    for attempt in $(seq 1 12); do
        if cargo publish --package "$crate"; then
            published=true
            break
        fi
        echo "Publish attempt ${attempt}/12 failed for ${crate}; waiting for the registry index" >&2
        sleep 15
    done

    if [ "$published" != true ]; then
        echo "Failed to publish ${crate} ${RELEASE_VERSION}" >&2
        exit 1
    fi

    # Wait until crates.io exposes this crate before publishing dependants.
    for attempt in $(seq 1 30); do
        if crate_exists "$crate"; then
            break
        fi
        if [ "$attempt" -eq 30 ]; then
            echo "Timed out waiting for ${crate} ${RELEASE_VERSION} on crates.io" >&2
            exit 1
        fi
        sleep 10
    done
done < "$packages"
