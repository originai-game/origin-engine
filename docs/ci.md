# CI and the local gate

The gate is one script, `scripts/check.sh`. CI runs it unchanged, so a green
local run predicts a green CI run on the same revision. The workflow
(`.github/workflows/ci.yml`) only provisions the machine.

## What runs where

| Step | Command | Local | CI `gate` (push to main, PR) | CI `perf-nightly` (03:17 UTC, manual) |
|---|---|---|---|---|
| `fmt` | `cargo fmt --all --check` | yes | yes | |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | yes | yes | |
| `test` | `cargo test --workspace` (logic and gfx tests in `og-tests`) | yes | yes, on lavapipe | |
| `wasm-build` | `cargo build --release --target wasm32-unknown-unknown -p <crate> --lib` for `scripts/wasm-crates.txt` | yes | yes | |
| `wasm-opt` | `wasm-bindgen --target web` (when the module uses it) then `wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int` | yes | yes | |
| `size-gate` | `scripts/size-gate.sh`: brotli `-q 11` sum ≤ `wasm_core_brotli_bytes` | yes | yes | |
| `drift` | `og api check` (generated `.d.ts`, wiki, JSON schema, Rust glue vs `crates/og-api/api.ogt`) | yes | yes | |
| `bun-test` | `bun run test` in each `packages/*` with a `test` script | yes | yes | |
| `determinism` | `tools/wasm-determinism/run.sh`: the same math digest and world hash on the host and on wasm32 scalar and simd128 under node | yes | yes | needs node |
| perf | `cargo test -p og-tests --release -- --ignored perf` | `make perf` | | yes |

CI runs on `ubuntu-latest`. The Rust toolchain comes from
`rust-toolchain.toml` (channel, rustfmt, clippy, the wasm target) through
`actions-rust-lang/setup-rust-toolchain`, which also caches `~/.cargo` and
`target/`. `scripts/install-tools.sh` installs wasm-bindgen-cli at the
version `Cargo.lock` pins, binaryen (`wasm-opt`), brotli and bun, skipping
any already present at the right version; the tools are cached under
`~/.local`. `mesa-vulkan-drivers` gives wgpu a Vulkan adapter (lavapipe) for
headless gfx tests.

On failure CI uploads `target/evidence/**` (gate summary, size report, gfx
actual and diff frames, perf reports), the wasm manifests and any
`*.diff.png` as the `evidence-<run>` artifact. The size report and summary
are uploaded on every run as `size-gate-<run>`.

## Running the gate locally

```
scripts/install-tools.sh             # once; macOS uses Homebrew for binaryen and brotli
scripts/check.sh                     # everything, stops at the first FAIL
scripts/check.sh clippy test         # selected steps, in the order given
scripts/check.sh wasm-build wasm-opt size-gate   # the size gate (make wasm)
scripts/check.sh --list
make check                           # the same through the Makefile
```

Each step prints `==> [n/N] step`, its own output, then `PASS`, `FAIL` or
`SKIPPED` with its duration; the script exits non-zero on the first `FAIL`
and writes the per-step lines to `target/evidence/check-summary.txt`. The
script honours `CARGO_TARGET_DIR` (agents working in parallel use their own,
e.g. `CARGO_TARGET_DIR=target/agent-<name>`) and exports `OG_EVIDENCE_DIR`
(default `<target>/evidence`) for test harnesses.

## Switches in `scripts/budgets.txt` and `scripts/wasm-crates.txt`

- `wasm-crates.txt` lists the crates the wasm step builds. `required` crates
  build in one cargo invocation and fail the gate on error; `pending` crates
  build one at a time and a failure prints `WARN` (use it while a crate is
  being ported; the step prints a `NOTE` once it builds). A crate builds a
  `.wasm` only with `crate-type = ["cdylib", …]`; until one does, the wasm
  steps report "no wasm artifact" and pass.
- `wasm_artifacts optional|required`: flip to `required` when `og-host-web`
  becomes a cdylib, so the size gate can no longer pass by measuring nothing.
- `drift_check optional|required`: `og api check` counts as available when
  `og --help` lists an `api` subcommand. While it is not, the step prints
  `SKIPPED: og api check not available yet`; with `required` the same
  situation fails the gate. Flip it in the commit that lands `og api check`.

## Updating a budget

Budgets come from `docs/design.md` §7 and live in `scripts/budgets.txt`.
Change the number in its own commit, with the measured value from
`target/evidence/size-gate.json` and the reason in the message; a raise above
a design §7 target also changes design §7 in the same commit. The core budget
is 4 MB brotli; the design's 6 MB with physics and animation becomes a second
key (e.g. `wasm_full_brotli_bytes`) when a feature-gated build exists.

## Re-capturing gfx references

gfx tests in `og-tests` render a scene headless and compare it with a
committed PNG (per-pixel tolerance plus an RMSE threshold, references per
backend tier). On mismatch they write `<scene>.actual.png` and
`<scene>.diff.png` under `$OG_EVIDENCE_DIR/gfx/`. When a rendering change is
intended:

```
OG_CAPTURE=1 cargo test -p og-tests          # rewrite the references
git diff --stat tests/                       # see which frames moved
```

Commit the new references with the change, and describe in the commit what
each moved frame now shows (AGENTS.md §Validation: a rendering change carries
a frame that was read and described). Capture on the backend tier the
reference belongs to; a lavapipe frame never replaces a Metal reference.

## When the size gate fails

1. Read the table `size-gate` printed (or `target/evidence/size-gate.json`
   from the CI artifact): which wasm grew and by how much.
2. Find the growth: `cargo tree -p og-host-web --target wasm32-unknown-unknown -e normal`
   for new dependencies, and `twiggy top target/og-wasm/opt/<name>.wasm`
   (`cargo install twiggy`) for the largest functions.
3. Prefer removing the cause: drop or feature-gate the dependency, disable
   default features, move debug-only code behind the non-`publish`
   configuration, avoid generic monomorphization blow-ups.
4. Only when the growth is intended and unavoidable, update the budget as
   above. Physics was expected to push past 4 MB; measured at 49ed935 it adds
   0.69 MB brotli (3D), 0.54 MB (2D) or 1.05 MB (both) to og-host-web's
   1.10 MB (`tests/evidence/M6/physics-wasm-size.json`), so it fits. If it
   ever does not, the answer is a feature-gated physics crate, not a budget
   raise (docs/phase-1.md §5).
