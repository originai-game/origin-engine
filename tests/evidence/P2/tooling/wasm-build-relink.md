# The gate's wasm-build relink

`scripts/check.sh` builds the web host twice for wasm32: the required set
(`-p og-render -p og-host-web`) and the publish variant
(`-p og-host-web --features og-host-web/publish`). With both in
`target/wasm32-unknown-unknown`, cargo 1.99 (one fingerprint per unit
whatever its features) found the other build's features each time and
recompiled og-probe, og-runtime and og-host-web on every run, even with
nothing changed.

Measured on the Apple M4 Pro, 2026-10-19, three alternating rounds of the
two commands as the gate runs them (`cargo build --profile wasm --target
wasm32-unknown-unknown --lib ...`):

| | required | publish |
|---|---|---|
| before (shared target dir), a rerun with nothing changed | 16.7 s | 12.2 s |
| after (publish in `target/wasm-publish`), first round | 16.0 s (catching up with the last shared build) | 205.8 s (cold: every dependency once, under load) |
| after, rounds 2 and 3 | 0.4 s, 0.5 s | 0.5 s, 0.4 s |

`packages/engine/scripts/build.sh` builds its publish wasm in the same
directory, so it reuses the gate's units.
