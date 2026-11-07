# Thin wrapper over scripts/; the scripts are the source of truth (docs/ci.md).
.PHONY: check quick fmt clippy test test-quick wasm size drift bun-test bun-quick tools perf web-reference list

check:      ; scripts/check.sh
quick:      ; scripts/check.sh quick
fmt:        ; scripts/check.sh fmt
clippy:     ; scripts/check.sh clippy
test:       ; scripts/check.sh full
test-quick: ; scripts/check.sh quick
wasm:       ; scripts/check.sh wasm-bindgen
size:       ; bash scripts/release-wasm.sh
drift:      ; scripts/check.sh drift
bun-test:   ; scripts/check.sh bun-test
bun-quick:  ; scripts/check.sh bun-quick
tools:      ; scripts/install-tools.sh
perf:       ; scripts/perf.sh
# Retained engine and fixture frames in headless Chrome (docs/ci.md); needs Chrome.
# Path-aware like the lanes: skipped when its inputs did not change (scripts/check.sh --all web-reference runs it).
web-reference: ; scripts/check.sh web-reference
list:       ; scripts/check.sh --list
