# scripts/check.sh (Tools M6)

```
PASS fmt (1s)
PASS clippy (3s)
PASS test (128s)
PASS wasm-build (17s)
PASS wasm-opt (20s)
PASS size-gate (16s)
PASS drift (1s)
PASS bun-test (64s)
PASS determinism (53s)
```

Size gate: og_host_web.wasm 8,044,057 bytes after wasm-opt, 2,190,006 brotli-q11 (52% of 4,194,304).
Also: cargo clippy -p og-host-web --target wasm32-unknown-unknown --all-targets -- -D warnings clean.
