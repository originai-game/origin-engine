# M2: the hello world on both hosts (Tools workstream, 2026-10-17)

**Scenario.** `packages/engine/examples/hello.ts`, the design's §3.2 example,
unchanged: its own `spawn_wave` command spawns 10,000 entities (named, with
Rng positions, `Velocity` and `Health`), then 600 fixed ticks of its `Move`
system over typed views of engine memory.

**Hashes after 600 ticks** (world hash, FNV-1a of the og-ecs snapshot):

| Host | Where | Hash |
|---|---|---|
| web (`og-host-web`, wasm in the engine Worker) | Chrome 150 headless, WebGPU, COOP/COEP, input over the SharedArrayBuffer ring | `3eb4283597417a0a` |
| web | Chrome 150, WebGL2 fallback | `3eb4283597417a0a` |
| web | Chrome 150 without cross-origin isolation (input over postMessage) | `3eb4283597417a0a` |
| web | the same wasm module under bun 1.4.2 | `3eb4283597417a0a` |
| native (`og-host-node`, napi addon, macOS arm64) | `og run --headless` (bun 1.4.2) | `3eb4283597417a0a` |
| native | the runner under node 26.3.0 | `3eb4283597417a0a` |

At load (before the spawn) both hosts hash `5b1f5e65486966cf`; another seed
gives another hash (`tests/hosts.test.ts`). Chrome rendered frames on the
OffscreenCanvas (WebGPU and WebGL2 tiers); there is nothing but the
background to draw, since `og.Mesh` cannot register until og-core has a
one-byte field type.

**View benchmark** (`packages/engine/bench/views-best.ts`, best of 7 fresh
processes, 2000 ticks each, 10,000 entities; ms per tick). "Raw loop" is
`tools/bench/views.ts`'s integrate over typed arrays; "engine tick" is one
whole `advance` tick of the hello world (dispatch, query, column views over
engine memory, the Move body, the og-ecs step); "Move call" is the system
call alone as the engine timed it.

| Variant | Raw loop | Engine tick | Tick / raw | Move call | Call / raw |
|---|---|---|---|---|---|
| node host, aliased views (bun) | 0.0123 | 0.0182 | 1.48× | 0.0145 | 1.18× |
| node host, copied views (bun) | 0.012 | 0.0334 | 2.78× | 0.0288 | 2.4× |
| web host, wasm memory (bun) | 0.0123 | 0.0248 | 2.02× | 0.0156 | 1.27× |
| node host, aliased views (node) | 0.0204 | 0.0243 | 1.19× | 0.0206 | 1.01× |
| web host in Chrome 150 (Worker, WebGPU, isolated) | 0.0253 | 0.0388 | 1.53× | | |

The spike's own run of `tools/bench/views.ts` on this machine today:
batch 0.0149 ms per tick.

Notes: copied views (`OG_NODE_VIEWS=copy`, the fallback where a runtime
refuses external ArrayBuffers) are measured but not used by default; node 26
and bun 1.4 both accept aliased external buffers. The machine was shared with
other build agents; the best-of runs are the least disturbed samples.
