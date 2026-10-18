# The player shell

Everything in an exported game page that is not the game (docs/web-export.md
is the full contract). `og build` inlines the boot markup, `brand.ts`'s
marks and the page configuration into `index.html` and bundles these
modules next to the engine Worker.

| File | Role |
|---|---|
| `page.ts` | the main thread (`play.js`): boot splash and real progress, the play gate (audio unlock, fullscreen, wake lock), letterbox and safe area, the engine Worker, the audio worklet, the Player link and the platform bridge |
| `player.ts` | Player protocol 2, game side: `hello`, `loading`, `ready` on the first drawn frame, `fail`, `crash`, `sample`, `event`; hears `hello`, `start`, `gesture`, `pause`, `resume`; `request(method, args)` sends a `call` and resolves with its `result` (matched by id) |
| `platform.ts` | `og.platform` on the web: the Worker's platform calls answered through the Player (`save.get`/`save.set` hold the store, `leaderboard.submit`/`top`, telemetry as `event`s), held until the Player's `hello` when framed, and the browser's storage (`src/platform-local.ts`) where there is no Player or no such method; what protocol 2 lacks is `docs/platform-asks.md` |
| `wire.ts` | the platform's `og-probe` wire (probe protocol 5) over the engine's in-page probe, origin-checked |
| `sw.ts` | the publish build's service worker: the whole build precached under its content hash |
| `brand.ts`, `brand/`, `shell.css` | the brand marks, motion and tokens (vendored from the platform's `packages/brand`) |

Rules: post only to `window.parent` with this page's own origin, hear only
`window.parent` at that origin; game code never reaches the DOM, a cookie
or another origin (it asks the engine, and the shell answers). Tests:
`tests/web/player.ts` (protocol 2 and the platform bridge in headless
Chrome), `tests/web/wire.ts`, `tests/web/export.ts`, and
`tests/platform.test.ts` for the local platform.
