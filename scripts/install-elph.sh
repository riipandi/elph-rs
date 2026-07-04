#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Elph -- Install script
#
# Usage:
#   curl -fsSL https://elph.space/elph/install.sh | bash
#   curl -fsSL https://elph.space/elph/install.sh | bash -s -- --canary
#   curl -fsSL https://raw.githubusercontent.com/riipandi/elph/main/scripts/install-elph.sh | bash
#
# Options:
#   --version <tag>      Pin a specific version (default: latest)
#   --canary             Use the canary release (pre-release)
#   --home <dir>         elph home directory (default: ~/.elph)
#   --install-dir <dir>  Binary install directory (default: ~/.local/bin)
#   --dry-run            Print what would happen without downloading
#   --help               Show this help
#
# Also via env vars: ELPH_VERSION, ELPH_CANARY, ELPH_HOME, ELPH_INSTALL_DIR
set -euo pipefail
