# Basis Universal transcoder (vendored)

`basis_transcoder.js` and `basis_transcoder.wasm` are Binomial LLC's Basis
Universal transcoder (https://github.com/BinomialLLC/basis_universal), the
Emscripten build of its `webgl/transcoder`, unmodified, copied from the npm
package `three@0.180.0` (`examples/jsm/libs/basis/`). That build includes
UASTC HDR support (the 1.50-series transcoder); the exact upstream tag is not
recorded by three. Licence: Apache-2.0 (`LICENSE`), attribution in `NOTICE`.

| File | sha256 |
|---|---|
| `basis_transcoder.js` | `8478b5b6d6b74e7d3082b89f6417321d8d1dc0307f2b30d4484bb11b441696a1` |
| `basis_transcoder.wasm` | `6cf17dc889352c42e9acf8897107978d127005fe3386c36a0e3845e27967630a` |

The web host transcodes cooked KTX2 textures with it in the engine Worker
(`src/web/basis.ts`); native hosts use og-cook's `transcode` feature instead.
To update: copy both files from a newer release (or a newer `three`), update
this table, and run `bun test tests/basis.test.ts`.
