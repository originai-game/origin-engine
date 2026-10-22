# Install test in an InstaBox machine, 0.1.1 (2026-10-22)

The first run of `tests/install/instabox.ts` (docs/release.md §3 "In an
InstaBox machine", §4 R4 and D4): the install test of the published
0.1.1, from the registry, inside a machine of the development sandbox the
owner named the same day (dashboard.instabox.sh; E2B is production's
provider only). Run from an Amp orb with `INSTABOX_API_KEY` in the
environment:

```sh
bun tests/install/instabox.ts --version 0.1.1 --registry
```

| | |
|---|---|
| Machine | `m_agqwgdgvkb2jrxvwtzhgjg3qou` (`og-install-0-1-1-qavkdc`), from the `agent-browser` template, deleted after the pass |
| Spec | Linux x86_64, 4 vCPU, 7964 MiB, 20 GiB `/data` (the free plan's one spec) |
| Carried | node v22.23.3, git 2.47.3; bun 1.4.2 installed by the script; no GPU |
| Renderer | `none` (`og doctor`: no suitable graphics adapter; vulkan drivers could not be loaded) |
| Result | **PASS on linux-x64**, `run.ts` 14.3 s; the whole run 19.0 s including create, upload and delete |

The same lines as the linux-x64 container (`0.1.1.md`): `og test` of the
scaffolded game and of the `runner` archetype pass with `1 frame(s)
unchecked: no renderer here`, never a failure; `og check`, `og build
--publish`, the knowledge's `validate.ts`, `mount.ts --package` and
`--check` pass. The machine's log, abridged:

```
[instabox 0.2s] machine: Linux x86_64 | 4 | 7964 MiB | v22.23.3 | git version 2.47.3 | 1.4.2 | renderer: none
[instabox 1.2s] $ bun /data/home/og-install/run.ts --version 0.1.1 --host-only --registry
   [install 0.0s] work directory /tmp/og-install-lW3Yqx (linux-x64, from the registry)
   [install 0.0s] -- 1 create-game
   [install 0.4s] -- 2 bun install
   [install 1.3s] -- 3 og --version, og doctor
   [install 6.1s]    renderer: none no GPU adapter: Auto: No suitable graphics adapter found; ...
   [install 6.1s] -- 4 og check, og test, og build --publish
   [install 10.5s]    og test:   ~ smoke.png: no renderer, frame unchecked | og test: 2 suite(s) passed; 1 frame(s) unchecked
   [install 11.9s] -- 5 the runner archetype
   [install 14.0s] -- 6 knowledge: validate, mount --package, --check
   [install 14.3s] PASS on linux-x64
[instabox 16.9s] PASS in m_agqwgdgvkb2jrxvwtzhgjg3qou (agent-browser, registry)
[instabox 19.0s] machine m_agqwgdgvkb2jrxvwtzhgjg3qou deleted
```

What the account allows, read from `GET /v1/me`, `/v1/usage` and
`/v1/templates` the same day (docs/release.md §6 item 6): the free plan,
whose machines are created from the `agent-browser` template or a
snapshot only (a create from `universal` is refused), one spec, five
machines and five snapshots as the limit but one machine at a time in
practice, auto-pause after thirty idle minutes, no manual lifecycle. One
earlier machine, `origin-sandbox-eval` (suspended, from the evaluation
thread), was left as it was. The file API takes 10 MiB per write (the
script uploads tarballs in 8 MiB parts for `--from`); a foreground
command is capped at ten minutes and not streamed (the script runs
`run.ts` in the background and follows its log). The SDK is
`@insforge/instabox-sdk` 0.5.3 (`tests/install/package.json`).

Not run here: `--from <tarballs>` (the registry was the question, the
packages being published); the platform's snapshot of the version
(stage D2) is the platform's to make and check.
