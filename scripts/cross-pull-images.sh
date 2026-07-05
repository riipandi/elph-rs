#!/usr/bin/env bash
# Pre-pull ghcr.io/cross-rs images into local Docker cache.
# cross reuses these automatically on subsequent builds.
# Image catalog: https://github.com/orgs/cross-rs/packages?repo_name=cross
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/cross-host.sh
source "${root}/scripts/cross-host.sh"

if ! command -v docker >/dev/null; then
    echo "docker is required" >&2
    exit 1
fi

tag="${CROSS_IMAGE_TAG:-latest}"

# cross-rs images are linux/amd64 only; Apple Silicon / ARM64 hosts need --platform.
docker_platform=()
platform_note=""
if [[ -n "${CROSS_DOCKER_PLATFORM:-}" ]]; then
    docker_platform=(--platform "$CROSS_DOCKER_PLATFORM")
    platform_note="${CROSS_DOCKER_PLATFORM}"
elif [[ "$(cross_host_rust_arch)" == "aarch64" ]]; then
    docker_platform=(--platform linux/amd64)
    platform_note="linux/amd64"
fi

all_targets=()
while IFS= read -r _target; do
    [[ -n "$_target" ]] && all_targets+=("$_target")
done < <("${root}/scripts/cross-targets.sh")

docker_targets=()
for target in "${all_targets[@]}"; do
    if [[ "$(cross_tool_for "$target")" == "cross" ]] && cross_image_published "$target"; then
        docker_targets+=("$target")
    fi
done

skipped_targets=()
for target in "${all_targets[@]}"; do
    if [[ "$(cross_tool_for "$target")" == "cross" ]] && ! cross_image_published "$target"; then
        skipped_targets+=("$target")
    fi
done

banner="Cross-pull  tag:${tag}"
if [[ -n "$platform_note" ]]; then
    banner="${banner}  ·  ${platform_note}"
fi
cross_log_banner "$banner"
echo

if ((${#docker_targets[@]} == 0)); then
    echo "No cross-rs images needed on this host."
    exit 0
fi

_add_pull_tag() {
    local candidate="$1" existing
    for existing in "${pull_tags[@]:-}"; do
        [[ "$existing" == "$candidate" ]] && return 0
    done
    pull_tags+=("$candidate")
}

pull_image() {
    local target="$1"
    local image tag_used output
    pull_tags=()

    _add_pull_tag "$tag"
    _add_pull_tag latest

    for tag_used in "${pull_tags[@]}"; do
        image="ghcr.io/cross-rs/${target}:${tag_used}"
        if output=$(docker pull "${docker_platform[@]}" "$image" 2>&1); then
            if [[ "$tag_used" == "$tag" ]]; then
                printf '  %-34s  pulled\n' "$target"
            else
                printf '  %-34s  pulled (%s)\n' "$target" "$tag_used"
            fi
            return 0
        fi
    done

    printf '  %-34s  failed\n' "$target" >&2
    echo "$output" | tail -3 | sed 's/^/    /' >&2
    return 1
}

for target in "${docker_targets[@]}"; do
    pull_image "$target" || exit 1
done

if ((${#skipped_targets[@]} > 0)); then
    echo
    echo "Skipped"
    for target in "${skipped_targets[@]}"; do
        printf '  %-34s  no ghcr.io image\n' "$target"
    done
fi

echo
echo "Cached"
docker image ls --filter reference='ghcr.io/cross-rs/*' \
    --format '  {{.Repository}}:{{.Tag}}  {{.Size}}' | sort
