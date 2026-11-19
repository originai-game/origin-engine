# Basis Universal transcoder (vendored)

`basis_transcoder.js` and `basis_transcoder.wasm` are Binomial LLC's Basis
Universal transcoder (https://github.com/BinomialLLC/basis_universal), its
`webgl/transcoder` at tag `v1_50_0_2` (the release whose prebuilt files
`three@0.180.0` ships, which this directory held before), built by
`scripts/basis-transcoder.sh` with Emscripten 7.0.0. The build differs from
upstream's prebuilt one in its glue only: it never calls `eval()` or
`new Function()` (`-s DYNAMIC_EXECUTION=0 -s EMBIND_AOT=1`, what upstream's
later `NO_DYNAMIC_EXECUTION` option adds), so a page whose
Content-Security-Policy has no `'unsafe-eval'` loads it, it takes its wasm as
the bytes `src/web/basis.ts` hands it (`wasmBinary`), and it is an ES module
(`export default BASIS`) that `src/web/basis.ts` imports. The
transcoder's source and its formats are the tag's. Licence: Apache-2.0
(`LICENSE`), attribution in `NOTICE`.

| File | sha256 |
|---|---|
| `basis_transcoder.js` | `c7e973c188a4cb45672c50937d0db9f85931b913a21bf9b9b1b8b02d2feed69c` |
| `basis_transcoder.wasm` | `f593e2722eb16014a3959e1a7a41ac09419bb4bd23e09e34c5c4f9e6c2b94c25` |

The web host transcodes cooked KTX2 textures with it in the engine Worker
(`src/web/basis.ts`); native hosts use og-cook's `transcode` feature instead.
To update: change `UPSTREAM` (and `EMSCRIPTEN`) in
`scripts/basis-transcoder.sh`, run it, update this table, and run
`bun test tests/basis.test.ts tests/web-csp.test.ts`.
