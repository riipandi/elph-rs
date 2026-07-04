#!/usr/bin/env bash
# Pre-pull ghcr.io/cross-rs images into local Docker cache.
# cross reuses these automatically on subsequent builds.
set -euo pipefail

if ! command -v docker >/dev/null; then
  echo "docker is required" >&2
  exit 1
fi

cross_ver=""
if command -v cross >/dev/null; then
  cross_ver="$(cross --version 2>/dev/null | awk 'NR==1 {print $2}')"
fi
tag="${CROSS_IMAGE_TAG:-${cross_ver:-main}}"

# Docker targets only (macOS has no cross-rs image)
docker_targets=(
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
  x86_64-unknown-linux-musl
  aarch64-unknown-linux-musl
  x86_64-pc-windows-gnu
  aarch64-pc-windows-msvc
)

echo "Pulling cross-rs images (tag: ${tag})"
echo

for target in "${docker_targets[@]}"; do
  image="ghcr.io/cross-rs/${target}:${tag}"
  echo "=> ${image}"
  if docker pull "$image"; then
    continue
  fi
  if [[ "$tag" != "main" ]]; then
    fallback="ghcr.io/cross-rs/${target}:main"
    echo "   retry ${fallback}"
    docker pull "$fallback"
  else
    exit 1
  fi
done

echo
echo "Cached images:"
docker image ls 'ghcr.io/cross-rs/*' --format '  {{.Repository}}:{{.Tag}}  {{.Size}}'
