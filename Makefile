# Thin wrapper over scripts/; the scripts are the source of truth (docs/ci.md).
.PHONY: check fmt clippy test wasm size drift bun-test tools perf list

check:      ; scripts/check.sh
fmt:        ; scripts/check.sh fmt
clippy:     ; scripts/check.sh clippy
test:       ; scripts/check.sh test
wasm:       ; scripts/check.sh wasm-build wasm-opt size-gate
size: wasm
drift:      ; scripts/check.sh drift
bun-test:   ; scripts/check.sh bun-test
tools:      ; scripts/install-tools.sh
perf:       ; cargo test -p og-tests --release -- --ignored perf
list:       ; scripts/check.sh --list
