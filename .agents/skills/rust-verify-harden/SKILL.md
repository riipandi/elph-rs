---
name: rust-verify-harden
description: >-
    Run make check/lint/test (cargo check/clippy/test) and fix failures, then
    audit changed/related Rust code for memory/resource leaks, deadlock risk,
    and data races, applying fixes and perf improvements. Use when user asks to
    verify build quality gates, prep Rust code for merge/release, or audit
    concurrency/memory safety in a Rust codebase.
---

# Rust Verify & Harden

## Purpose

Run standard quality gates (check/lint/test), fix failures, then do a deeper concurrency/memory safety + perf pass on the affected Rust code.

## Workflow

### Phase 1 — Quality gates

1. Run `make check` (→ `cargo check`). If it fails, fix root cause, re-run until clean.
2. Run `make lint` (→ `cargo clippy --all-targets -- -D warnings`). Fix violations by changing code, not by adding `#[allow(...)]` unless clippy is genuinely wrong — justify in a comment if so.
3. Run `make test` (→ `cargo nextest run`). Fix failing tests (fix the bug, not the assertion, unless assertion is wrong). Re-run until clean.
4. If any target doesn't exist in Makefile, say so and skip — don't guess a replacement command.

### Phase 2 — Deep analysis (scope: files touched in phase 1 + their direct callers/callees)

1. **Memory/resource leaks**: `Rc<RefCell<T>>` / `Arc<Mutex<T>>` cycles preventing drop, missing `Weak` where a back-reference is needed, file/socket handles not dropped before a long-lived scope, `mem::forget`/`Box::leak` used without clear justification, unbounded `Vec`/`HashMap`/channel growth (esp. in long-running tasks/actors), tasks spawned via `tokio::spawn` never awaited/aborted (detached task leak).
2. **Deadlock risk**: nested `Mutex`/`RwLock` acquisition with inconsistent order across call sites, lock guard held across `.await` (check for `MutexGuard` not `Send`, or async mutex held over blocking work), lock held across a call into code that may re-acquire the same lock, `std::sync::Mutex` used inside async code instead of `tokio::sync::Mutex` where it blocks the executor.
3. **Race conditions / data races**: `unsafe impl Send`/`Sync` on types with interior mutability not actually thread-safe, raw pointer aliasing in `unsafe` blocks, `static mut`, non-atomic shared counters/flags that should be `Atomic*`, check-then-act on shared state without a single lock covering both steps. Run `cargo miri test` and, if the project has async/lock-heavy code, `cargo nextest run` under `loom` (if already a dev-dependency — don't add it unprompted).
4. Apply minimal fixes. One-line reasoning per fix (what was wrong, why fix is correct).

### Phase 3 — Perf pass (same scope)

1. Flag: unnecessary `.clone()`/allocations in hot paths, `String`/`Vec` reallocation from missing `with_capacity`, blocking I/O or `std::sync::Mutex` on the async hot path, O(n²) where O(n log n) is feasible, redundant serialization/deserialization, unnecessary `Box`/`dyn` indirection in tight loops, `collect()` into an intermediate `Vec` where an iterator chain would do.
2. Apply fix only if low-risk and localized. If it needs a design change (e.g. restructuring ownership), report it instead of forcing it in.

### Phase 4 — Close out

1. Re-run `make check`, `make lint`, `make test` — must all pass clean.
2. Summarize: files changed, issues found (by category), fixes applied, remaining known risks (if any weren't safe to auto-fix).

## Notes

- Never touch unrelated files just to "improve" them — scope is phase-1 output + direct dependencies.
- If a fix is risky/ambiguous (behavior change, unclear intent, ownership restructuring), report instead of applying.
- Don't add new dev-dependencies (`loom`, `miri` setup, etc.) unless already present or explicitly approved.
- Git commit/push only if explicitly instructed.

## Example

User: "run quality gates and audit concurrency issues in this project"
→ run phases 1–4, report summary with file:line refs for each issue found.
