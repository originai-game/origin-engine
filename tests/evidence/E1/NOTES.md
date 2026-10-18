# E1 evidence

One line per file: what it is, how it was produced, at which revision.

- `ui-kit-test-run.md` — the commands and results listed inside (cargo test/clippy/fmt, og api check, bun run test, og test templates/ui-kit, the browser procedure) (rev 3db4319+dirty, 2026-10-18, sha256 3e617e06d85b6e44)
- `ui-kit-desktop-sheet.png` — contact sheet of crates/og-ui/references/metal/kit-desktop-*.png, recorded by OG_UI_CAPTURE=1 og test templates/ui-kit (frames-desktop, 1280x720) (rev 3db4319+dirty, 2026-10-18, sha256 c2a2c1da399e2d0c)
- `ui-kit-phone-sheet.png` — contact sheet of crates/og-ui/references/metal/kit-phone-*.png, recorded by OG_UI_CAPTURE=1 og test templates/ui-kit (frames-phone, 0:viewport:phone, --size 1170x2532) (rev 3db4319+dirty, 2026-10-18, sha256 18886640c0178111)
- `ui-widget-set-metal.png` — cargo test -p og-ui --test kit_gfx (the frame it renders, equal to references/metal/widget-set.png) (rev 3db4319+dirty, 2026-10-18, sha256 b8eac6b81f46c4de)
- `ui-break-and-restore.md` — each listed break applied to one file, the named test run, the file restored (cargo test -p og-ui --test <t>; og test templates/ui-kit for the playtest rows) (rev 3db4319+dirty, 2026-10-18, sha256 a68258d0b061e981)
- `ui-kit-dump-ui-shop.json` — dump_ui at 1.6 s of templates/ui-kit/tests/touch.og.ts, after Buy: the shop dialog and the toast (rev 3db4319+dirty, 2026-10-18, sha256 bb80d5d5b4e992b4)
- `ui-kit-web-shop.png` — og build templates/ui-kit; drive.ts (M6 web procedure) in headless Chrome 150 WebGPU, tick 600 of the script in ui-kit-two-host-hash.json (rev 3db4319+dirty, 2026-10-18, sha256 2aa0d60bc655a025)
- `ui-kit-two-host-hash.json` — world hash at tick 600: og run --headless (native) and drive.ts (Chrome, WebGPU) with the same script (rev 3db4319+dirty, 2026-10-18, sha256 28e9549ba3e829a2)
- `vfx-feature-costs.md` — `og-gfx --costs` on metal: every effect's frame cost against features-off, and each E1 scene against itself without the effect (rev 3db4319+E1 tree, 2026-10-18)
- `vfx-break-each-once.json` — each effect broken once in its source and its effects test run, then restored; 24 of 24 fail as they must (the spawn-count test needed a 45/s emitter before it caught its break) (rev 3db4319+E1 tree, 2026-10-18)
- `web-wire-transcript.json` — `bun tests/web/wire.ts`: the og-probe wire of the runner's dev and publish builds from a cross-origin parent in headless Chrome, isolated and not; 120 checks (rev 3db4319+E1 tree, 2026-10-18)
- `web-player-result.json` — `bun tests/web/player.ts`: Player protocol 2 against the runner's publish build; 10 checks (a run under a loaded machine once failed `start plays`; idle, it passes) (rev 3db4319+E1 tree, 2026-10-18)
- `web-export-result.json`, `web-runner-webgpu-play.png`, `web-runner-webgl2-play.png`, `web-match3-phone-play.png`, `web-runner-boot-progress.png`, `web-runner-offline.png` — `bun tests/web/export.ts`: both archetypes exported with --publish, served with their `_headers`, on WebGPU and WebGL2, the boot splash with a real progress value, offline from the service worker, phone layout at 390x844@3; 40 checks (rev 3db4319+E1 tree, 2026-10-18)
