#!/usr/bin/env bash
# Build the full release/ bundle for eclaw and elph.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

cross="$(command -v cross || true)"
cargo="$(command -v cargo || true)"
stage="${root}/scripts/cross-stage.sh"

if [[ -z "$cross" || -z "$cargo" ]]; then
  echo "cross and cargo are required; run: make prepare" >&2
  exit 1
fi

# Linux (glibc + Pi 3 32-bit), Alpine (musl), Windows — via cross + Docker
docker_targets=(
  x86_64-unknown-linux-gnu      # Ubuntu/Debian x86_64
  aarch64-unknown-linux-gnu     # Raspberry Pi OS 64-bit (Pi 3/4/5)
  armv7-unknown-linux-gnueabihf # Raspberry Pi OS 32-bit (Pi 3)
  x86_64-unknown-linux-musl     # Alpine x86_64
  aarch64-unknown-linux-musl    # Alpine ARM64
  x86_64-pc-windows-gnu
  aarch64-pc-windows-msvc
)

darwin_targets=(
  x86_64-apple-darwin
  aarch64-apple-darwin
)

build_and_package() {
  local tool="$1"
  local target="$2"
  local pkg="$3"

  "$tool" build --release -p "$pkg" --target "$target"
  "$stage" "$target" "$pkg"
}

for target in "${docker_targets[@]}"; do
  build_and_package "$cross" "$target" eclaw
  build_and_package "$cross" "$target" elph
done

if [[ "$(uname -s)" == "Darwin" ]]; then
  for target in "${darwin_targets[@]}"; do
    build_and_package "$cargo" "$target" eclaw
    build_and_package "$cargo" "$target" elph
  done
else
  echo "Skipped macOS targets (requires a Mac host)" >&2
fi