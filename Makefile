# Thin wrapper over scripts/; the scripts are the source of truth (docs/ci.md).
.PHONY: check quick fmt clippy test test-quick wasm size drift bun-test bun-quick determinism tools perf web-reference list

check:      ; scripts/check.sh
quick:      ; scripts/check.sh quick
fmt:        ; scripts/check.sh fmt
clippy:     ; scripts/check.sh clippy
test:       ; scripts/check.sh test
test-quick: ; scripts/check.sh test-quick
wasm:       ; scripts/check.sh wasm-build wasm-opt size-gate
size: wasm
drift:      ; scripts/check.sh drift
bun-test:   ; scripts/check.sh bun-test
bun-quick:  ; scripts/check.sh bun-quick
determinism: ; scripts/check.sh determinism
tools:      ; scripts/install-tools.sh
perf:       ; cargo test -p og-tests --release -- --ignored perf
# The webgpu and webgl2 tiers of templates/fx-showcase in headless Chrome (docs/ci.md); needs Chrome.
# Path-aware like the lanes: skipped when its inputs did not change (scripts/check.sh --all web-reference runs it).
web-reference: ; scripts/check.sh web-reference
list:       ; scripts/check.sh --list
