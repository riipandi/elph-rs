#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# eclaw -- Install script
#
# Usage:
#   curl -fsSL https://elph.space/eclaw/install.sh | bash
#   curl -fsSL https://elph.space/eclaw/install.sh | bash -s -- --canary
#   curl -fsSL https://raw.githubusercontent.com/riipandi/elph/main/scripts/install-eclaw.sh | bash
#
# Options:
#   --version <tag>      Pin a specific version (default: latest)
#   --canary             Use the canary release (pre-release)
#   --home <dir>         eclaw home directory (default: ~/.eclaw)
#   --install-dir <dir>  Binary install directory (default: ~/.local/bin)
#   --dry-run            Print what would happen without downloading
#   --help               Show this help
#
# Also via env vars: ECLAW_VERSION, ECLAW_CANARY, ECLAW_HOME, ECLAW_INSTALL_DIR
set -euo pipefail
