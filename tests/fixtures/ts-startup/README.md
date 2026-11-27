# TS startup acceptance

This fixture exercises the real project bootstrap, imported helper dependencies, two content namespaces, typed field cook, scanned scene catalogue, and native startup. It is not a game template. `og-project.json` is ordinary authored data and check/cook must not rewrite it. Scene authoring remains the existing cooker until P3; the fixture will move with that cutover.

From the repository root, with the engine package and native addon built:

```sh
OG_ENGINE_PACKAGE="$PWD/packages/engine" OG_BIN="$PWD/target/debug/og" target/debug/og check tests/fixtures/ts-startup --json
OG_ENGINE_PACKAGE="$PWD/packages/engine" OG_BIN="$PWD/target/debug/og" target/debug/og cook tests/fixtures/ts-startup --json
OG_BIN="$PWD/target/debug/og" OG_NATIVE_ADDON=/path/to/og_host_node.node bun tests/support/ts-startup.ts
```

Cook outputs: `base/fields/depth`, `island/fields/slope`, `base/scenes/harbor`, `base/scenes/inland`. The first field imports an asymmetric sample array from `lib/`; the island module is independent of the base source namespace.

The script advances 15 ticks and sends this exact probe request:

```json
{"jsonrpc":"2.0","id":1,"method":"state","params":{}}
```

Expected facts: `result.tick = 15`, `result.seed = 83`, entities includes `{"id":"harbor-marker"}` and excludes `inland-marker`. Project startup is `tick: 30`, `scene: "harbor"`, `saveVersion: 4`; `og.Time.rate = 30`, elapsed is 0.5 s. Snapshot restores on the same version; the default save version 1 rejects it. Runtime warning log is empty. The script asserts these facts rather than relying on a successful exit alone.

This is a no-render native functional acceptance. It does not check GPU quality, web/wasm or Hermes parity.
