#!/usr/bin/env bash
# E2E CLI smoke tests for Owly (no LLM unless OWLY_E2E_LLM=1).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
OWLY="${OWLY_BIN:-$ROOT/target/release/owly}"
REAL_HOME="${HOME:-}"
TMP="$(mktemp -d)"
export HOME="$TMP/home"
mkdir -p "$HOME"
CREDENTIALS_HOME="${OWLY_CREDENTIALS_HOME:-$REAL_HOME}"
PASS=0
FAIL=0
SKIP=0
LAST_OUT=""

run_expect() {
    local name="$1"
    local expect_exit="$2"
    shift 2
    local -a args=()
    if (($# > 0)); then
        args=("$@")
    fi
    set +e
    local out
    if ((${#args[@]} > 0)); then
        out="$("$OWLY" "${args[@]}" 2>&1)"
    else
        out="$("$OWLY" 2>&1)"
    fi
    local code=$?
    set -e
    if [[ "$code" -eq "$expect_exit" ]]; then
        PASS=$((PASS + 1))
        printf '  ✓ %s\n' "$name"
    else
        FAIL=$((FAIL + 1))
        printf '  ✗ %s (expected exit %s, got %s)\n' "$name" "$expect_exit" "$code"
        printf '    cmd: owly %s\n' "${args[*]}"
        printf '    out: %s\n' "$(echo "$out" | head -3 | tr '\n' ' ')"
    fi
    LAST_OUT="$out"
}

run_expect_output() {
    local name="$1"
    local expect_exit="$2"
    local pattern="$3"
    shift 3
    if (($# > 0)); then
        run_expect "$name" "$expect_exit" "$@"
    else
        run_expect "$name" "$expect_exit"
    fi
    if echo "$LAST_OUT" | grep -qE "$pattern"; then
        PASS=$((PASS + 1))
        printf '  ✓ %s output matches /%s/\n' "$name" "$pattern"
    else
        FAIL=$((FAIL + 1))
        printf '  ✗ %s output missing /%s/\n' "$name" "$pattern"
        printf '    out: %s\n' "$(echo "$LAST_OUT" | head -5 | tr '\n' ' ')"
    fi
}

skip() {
    SKIP=$((SKIP + 1))
    printf '  ○ %s (skipped)\n' "$1"
}

if [[ ! -x "$OWLY" ]]; then
    echo "Missing binary: $OWLY (run: cargo build -p owly --release)" >&2
    exit 1
fi

echo "Owly E2E CLI tests"
echo "  binary: $OWLY"
echo "  HOME:   $HOME"
echo

echo "== Core =="
run_expect_output "bare owly" 0 "Interactive mode not yet implemented"
run_expect_output "--help" 0 "owly personal" --help
run_expect_output "-h alias" 0 "owly personal" -h
run_expect_output "--credentials" 0 "credential diagnostics" --credentials
run_expect_output "--init without mode" 1 "requires a mode" --init
run_expect_output "--init and --update" 1 "not both" --init --update
run_expect_output "-p without message" 1 "requires a message" -p
run_expect_output "invalid --mode" 1 "Invalid --mode" --mode bogus --dry-run

echo "== Flags (trailing recovery + dry-run) =="
run_expect_output "--stream trailing on dry-run chat" 0 "action:.*chat" --dry-run personal "hello" --stream
run_expect_output "--verbose trailing on dry-run update" 0 "action:.*update" personal --update --dry-run --verbose
run_expect_output "--modelId trailing dry-run init" 0 "model:.*big-pickle" personal --init --dry-run --modelId big-pickle
run_expect_output "--model= trailing dry-run" 0 "model:" --dry-run --mode code --init --model=big-pickle
run_expect_output "--dry-run --mode personal init" 0 "mode:.*personal" --dry-run --mode personal --init
run_expect_output "personal --update --dry-run" 0 "action:.*update" personal --update --dry-run

echo "== Dry-run =="
run_expect_output "dry-run personal init" 0 "action:.*init" --dry-run personal --init
run_expect_output "dry-run personal update" 0 "action:.*update" --dry-run personal --update
run_expect_output "dry-run personal chat" 0 "action:.*chat" --dry-run "hello"
run_expect_output "dry-run code init" 0 "mode:.*code" --dry-run code --init
run_expect_output "dry-run code update" 0 "mode:.*code" --dry-run code --update
run_expect_output "dry-run --mode code" 0 "mode:.*code" --dry-run --mode code --update
run_expect_output "personal --init --dry-run" 0 "action:.*init" personal --init --dry-run
run_expect_output "code --update --dry-run" 0 "action:.*update" code --update --dry-run
run_expect_output "--dry-run personal --init" 0 "action:.*init" --dry-run personal --init

echo "== Product subcommands =="
run_expect_output "auth list" 0 "auth configure" auth list
run_expect_output "auth configure missing provider" 1 "Usage:" auth configure
run_expect_output "auth tools rejected" 1 "not supported" auth tools notion
run_expect_output "auth slack rejected" 1 "not supported" auth slack
run_expect_output "auth gmail rejected" 1 "not supported" auth gmail
run_expect_output "auth unknown provider" 1 "Unknown auth command" auth bogus
run_expect_output "ngrok rejected" 1 "not supported" ngrok
run_expect_output "cron list" 0 "schedule" cron list
run_expect_output "cron pause missing target" 1 "Usage: owly cron pause" cron pause
run_expect_output "cron resume missing target" 1 "Usage: owly cron resume" cron resume
run_expect_output "cron delete missing target" 1 "Usage:" cron delete
run_expect_output "cron pause unknown target" 1 "Unknown schedule target" cron pause bogus
run_expect_output "cron pause all (no schedules)" 1 "No schedules configured" cron pause all
run_expect_output "ingest invalid target" 1 "ingestion" ingest bogus
run_expect_output "ingest all (no sources)" 1 "No configured ingestion" ingest all
run_expect_output "ingest unknown flag" 1 "Unknown option" ingest all --bogus

echo "== Auth configure (isolated HOME) =="
run_expect_output "auth configure git-repo" 0 "Connector config" auth configure git-repo
run_expect_output "auth configure web-search" 0 "Connector config" auth configure web-search
run_expect_output "auth configure hackernews" 0 "Connector config" auth configure hackernews
run_expect_output "auth configure git-repo exists" 0 "exists" auth configure git-repo
run_expect_output "auth configure git-repo --force" 0 "updated" auth configure git-repo --force

echo "== Cron lifecycle (after auth configure) =="
run_expect_output "cron pause git-repo" 0 "Paused schedule" cron pause git-repo
run_expect_output "cron list shows paused git-repo" 0 "git-repo.*paused" cron list
run_expect_output "cron resume git-repo" 0 "Resumed schedule" cron resume git-repo
run_expect_output "cron delete git-repo" 0 "Deleted schedule" cron delete git-repo

echo "== Ingest (configured connector, no LLM wiki write) =="
run_expect_output "ingest all after configure" 0 "wiki: skipped" ingest all
run_expect_output "ingest git-repo" 0 "Owly ingest git-repo" ingest git-repo
run_expect_output "ingest --print flag accepted" 0 "Owly ingest" ingest git-repo --print

echo "== Code mode paths =="
CODE_REPO="$TMP/repo"
mkdir -p "$CODE_REPO"
run_expect_output "code dry-run init in empty repo" 0 "wiki:" --directory "$CODE_REPO" --dry-run code --init
run_expect_output "--directory trailing dry-run" 0 "wiki:" code --init --dry-run --directory "$CODE_REPO"

if [[ "${OWLY_E2E_LLM:-}" == "1" ]]; then
    echo "== LLM (live) =="
    if [[ -f "$CREDENTIALS_HOME/.owly/.env" ]]; then
        set -a
        # shellcheck disable=SC1090
        source "$CREDENTIALS_HOME/.owly/.env"
        set +a
    fi
    run_expect_output "personal chat stream" 0 "Owly Chat" "hi"
    run_expect_output "personal chat print" 0 "Owly Chat" -p "say ok"
    run_expect_output "personal trailing flags chat" 0 "Owly Chat" personal -p "say ok"
    run_expect_output "personal chat verbose" 0 "Owly Chat" -v "say ok"
    run_expect_output "personal positional dry-run then stream" 0 "Owly Chat" personal "ping"
else
    skip "personal chat stream (set OWLY_E2E_LLM=1)"
    skip "personal chat print (set OWLY_E2E_LLM=1)"
    skip "personal trailing flags chat (set OWLY_E2E_LLM=1)"
    skip "personal chat verbose (set OWLY_E2E_LLM=1)"
    skip "personal positional chat (set OWLY_E2E_LLM=1)"
fi

echo
echo "Results: $PASS passed, $FAIL failed, $SKIP skipped"
if [[ "$FAIL" -gt 0 ]]; then
    exit 1
fi
