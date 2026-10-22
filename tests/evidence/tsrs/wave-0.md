# Wave 0 — measurements (goals/tsrs/plan.md §五)

Tree `684db7cb` (main, the plan's commit) plus the Wave 0 files; 2026-10-22;
an Amp orb: Linux x86_64, Intel Xeon @ 2.60 GHz, `nproc` 8, 15 GB, load
0.66 / 0.90 / 0.43 when the table started, nothing else running.
tsgo is the native binary of `typescript@7.0.2`
(`node_modules/@typescript/typescript-linux-x64/lib/tsc`, no JS launcher);
tsrs is the one `scripts/install-tools.sh` installs.

## 0.1 The pinned tsrs

| | |
|---|---|
| package | `@tsrs/linux-x64-gnu@0.18.0-ts7.1.0-dev.20260929` |
| `dist.integrity` (npm) | `sha512-EdBwBgNICGu1r1h23ixdcKwsMo6E9ll7ZsY3mNERNYqE1Pwdtcat99d5E7zB1J+oy/Xk6YqiizaB6nolIjpDTw==` |
| sha512 of the tarball downloaded | the same (the install step compares them and stops on a difference) |
| `tsrs --version` | `Version 7.1.0-dev.20260930.4 (tsrs 0.18.0-ts7.1.0-dev.20260929, microsoft/TypeScript@b85298b6a81f)` |
| binary | 28,708,944 bytes |
| `lib.*.d.ts` (111 files) | 3,790,793 bytes |
| installed directory (`~/.local/lib/tsrs-<ver>/`) | 32 MB (npm `unpackedSize` 32,561,296) |

The other four packages' `dist.integrity`, from `npm view` the same day,
are in `scripts/install-tools.sh` (linux-arm64-gnu 39.2 MB, darwin-arm64
21.3 MB, linux-x64-musl 23.3 MB, linux-arm64-musl 20.8 MB unpacked).
The version string carries the package version, so `og doctor` compares
it with `TSRS_VERSION` directly.

## 0.2 Equal-thread benchmark

`goals/tsrs/evidence/bench.sh` (5 runs a cell; wall clock around the child
process, peak RSS from its own rusage). `-p <config> --noEmit
--incremental false --pretty false --checkers N`, both tools. All targets
check clean on both. `templates/game` is checked with the `paths` og
check gives it in the repository; "startup" is one file with no imports.

| target | checkers | tsgo ms (range) | tsgo MB (range) | tsrs ms (range) | tsrs MB (range) |
|---|---|---|---|---|---|
| engine:types | 1 | 90 (84–114) | 38 (37–38) | 32 (32–39) | 22 (22–22) |
| engine:types | 2 | 93 (82–105) | 39 (39–39) | 32 (30–37) | 24 (24–25) |
| engine:types | 4 | 74 (73–81) | 41 (41–42) | 35 (30–37) | 26 (24–29) |
| engine:types | 8 | 76 (73–82) | 44 (43–44) | 30 (29–31) | 28 (28–29) |
| engine:runtime | 1 | 1082 (1068–1101) | 218 (217–220) | 334 (326–358) | 89 (89–91) |
| engine:runtime | 2 | 892 (871–915) | 234 (225–239) | 204 (201–221) | 99 (95–101) |
| engine:runtime | 4 | 715 (693–774) | 252 (251–254) | 140 (130–185) | 106 (106–107) |
| engine:runtime | 8 | 668 (644–759) | 282 (269–293) | 125 (122–130) | 126 (124–127) |
| engine:templates | 1 | 385 (365–416) | 79 (79–80) | 127 (121–135) | 46 (40–46) |
| engine:templates | 2 | 320 (306–362) | 88 (86–89) | 84 (76–88) | 44 (41–48) |
| engine:templates | 4 | 253 (239–270) | 96 (95–97) | 56 (54–56) | 46 (46–48) |
| engine:templates | 8 | 217 (207–235) | 107 (104–108) | 47 (46–50) | 49 (47–50) |
| templates/game | 1 | 88 (88–90) | 38 (38–39) | 34 (34–38) | 23 (23–24) |
| templates/game | 2 | 82 (79–92) | 40 (40–40) | 30 (29–31) | 25 (24–25) |
| templates/game | 4 | 81 (74–86) | 42 (41–42) | 30 (29–32) | 28 (27–31) |
| templates/game | 8 | 76 (75–95) | 44 (44–44) | 30 (30–31) | 29 (29–29) |
| examples/brasswick | 1 | 156 (154–167) | 49 (49–50) | 54 (54–56) | 28 (28–28) |
| examples/brasswick | 2 | 138 (135–155) | 51 (51–52) | 44 (41–48) | 28 (27–29) |
| examples/brasswick | 4 | 124 (119–136) | 55 (54–56) | 40 (35–43) | 34 (31–35) |
| examples/brasswick | 8 | 119 (116–126) | 61 (60–61) | 35 (35–38) | 42 (38–45) |
| examples/hollowbell | 1 | 172 (170–177) | 50 (50–51) | 62 (59–74) | 28 (28–28) |
| examples/hollowbell | 2 | 150 (146–153) | 53 (53–54) | 52 (45–56) | 28 (28–30) |
| examples/hollowbell | 4 | 124 (121–128) | 57 (57–58) | 38 (36–47) | 34 (33–37) |
| examples/hollowbell | 8 | 127 (114–127) | 64 (63–64) | 37 (37–37) | 42 (40–43) |
| examples/kestrel-stair | 1 | 204 (198–212) | 55 (55–55) | 64 (63–66) | 28 (28–31) |
| examples/kestrel-stair | 2 | 155 (153–159) | 59 (58–59) | 46 (45–46) | 29 (28–31) |
| examples/kestrel-stair | 4 | 144 (139–154) | 64 (63–65) | 41 (39–44) | 35 (35–36) |
| examples/kestrel-stair | 8 | 133 (125–135) | 70 (70–71) | 37 (36–40) | 46 (46–48) |
| examples/lantern-vale | 1 | 139 (136–139) | 47 (46–48) | 50 (48–52) | 27 (27–27) |
| examples/lantern-vale | 2 | 116 (114–118) | 49 (49–49) | 37 (37–38) | 26 (25–27) |
| examples/lantern-vale | 4 | 105 (102–113) | 52 (51–53) | 34 (34–36) | 34 (33–34) |
| examples/lantern-vale | 8 | 103 (103–109) | 58 (56–58) | 35 (35–39) | 40 (37–41) |
| examples/meridian-loop | 1 | 179 (166–196) | 50 (50–50) | 56 (56–59) | 28 (28–28) |
| examples/meridian-loop | 2 | 150 (144–160) | 54 (53–55) | 44 (43–49) | 28 (28–29) |
| examples/meridian-loop | 4 | 126 (119–138) | 58 (57–58) | 38 (36–41) | 37 (35–38) |
| examples/meridian-loop | 8 | 124 (121–132) | 64 (62–64) | 33 (33–35) | 44 (43–46) |
| startup (1 file) | 1 | 49 (49–62) | 29 (29–29) | 20 (20–22) | 18 (17–18) |
| startup (1 file) | 2 | 49 (47–51) | 30 (30–31) | 21 (20–23) | 18 (18–18) |
| startup (1 file) | 4 | 50 (47–53) | 32 (32–32) | 23 (22–24) | 20 (19–20) |
| startup (1 file) | 8 | 52 (50–55) | 34 (34–35) | 26 (24–27) | 24 (23–25) |

At `--checkers 4` (og's default on this machine) tsrs takes 20–47 % of
tsgo's wall clock on every target (engine:runtime 140 ms against 715 ms,
the examples 34–41 ms against 105–144 ms) and 42–67 % of its peak memory.
Startup is 23 ms against 50 ms. tsrs's own claim of "up to 8x" is not
reached here (best 5.1x, engine:runtime at 4 checkers); its single-thread
lead is already 2.5–3.3x, so most of the gain is per-checker, not scaling.
Above 4 checkers neither tool gains more than 15 % on the examples, which
supports D6's `min(4, cores)`.

## 0.3 Conformance corpus

Clean projects: the nine configs of `scripts/tsrs-diff.sh` (the engine's
three, templates/game, the five examples) are clean on both checkers,
tsgo 7.0.2 and 7.1.0-dev.20260930.4 alike.

`goals/tsrs/evidence/corpus/`: 22 files, one error each a game author
makes, with the template's options and `paths` to the engine package
(`tsconfig.json`). `scripts/tsrs-diff.sh goals/tsrs/evidence/corpus/tsconfig.json`:

| file | code | tsgo 7.0.2 | tsrs | 7.1-dev |
|---|---|---|---|---|
| 01 a component field given the wrong type (`hp: "full"`) | 2322 | ✓ | ✓ | ✓ |
| 02 `defineComponent` without its import | 2304 | ✓ | ✓ | ✓ |
| 03 `exactOptionalPropertyTypes`: `seed: undefined` in a PlaytestDesc (3-line message) | 2379 | ✓ | ✓ | ✓ |
| 04 `verbatimModuleSyntax`: `World` imported as a value | 1484 | ✓ | ✓ | ✓ |
| 05 a specifier `paths` does not resolve (`@origingame/engin`) | 2307 | ✓ | ✓ | ✓ |
| 06 `defineSystem` reads a string instead of a component | 2322 | ✓ | ✓ | ✓ |
| 07 a `Vector3` given as `[0, 1, 0]` | 2739 | ✓ | ✓ | ✓ |
| 08 playtest `t.ok` without its message | 2554 | ✓ | ✓ | ✓ |
| 09 `noFallthroughCasesInSwitch` | 7029 | ✓ | ✓ | ✓ |
| 10 a `Result` read without checking `ok` (2-line message) | 2339 | ✓ | ✓ | ✓ |
| 11 an untyped parameter under `strict` | 7006 | ✓ | ✓ | ✓ |
| 12 `defineSystem` without its body | 2554 | ✓ | ✓ | ✓ |
| 13 an unknown `SystemDesc` field (`order`) | 2353 | ✓ | ✓ | ✓ |
| 14 assigning `System.name` (readonly) | 2540 | ✓ | ✓ | ✓ |
| 15 `og.i18n.t` (a string) into a number | 2322 | ✓ | ✓ | ✓ |
| 16 a declared return type not returned | 2322 | ✓ | ✓ | ✓ |
| 17 `allowImportingTsExtensions`: `./not-here.ts` | 2307 | ✓ | ✓ | ✓ |
| 18 an unused `@ts-expect-error` | 2578 | ✓ | ✓ | ✓ |
| 19 `await` in a function that is not async | 1308 | ✓ | ✓ | ✓ |
| 20 a block-scoped name declared twice (two diagnostics) | 2451 | ✓ | ✓ | ✓ |
| 21 `og.game({ systems: [Health] })` | 2739 | ✓ | ✓ | ✓ |
| 22 a `Vector3` field value without `z` | 2741 | ✓ | ✓ | ✓ |

23 diagnostics from tsgo; tsrs reports the same 23 (file, line, col, code)
keys with byte-identical messages, continuation lines included: **zero
misses, zero extras, zero message differences**, so nothing needed the
7.1-dev re-check to be classified (it agrees with both). The comparer
itself fails on a miss: with a stand-in tsrs that prints nothing it reports
`MISS` for all 23 and exits 1.

Exit status: on the same diagnostics tsrs exits 2 where tsgo exits 1
(`templates/game` without `paths`: 51 identical lines, tsrs 2, tsgo 1); a
broken tsconfig.json is 2 on tsrs as well (TypeScript's 3 never appeared).
og reads 1–4 alike as "finished, with diagnostics" (§4.2).

## 0.4 Go / no-go

Against the thresholds of §六:

| threshold | measured | |
|---|---|---|
| zero misses on the corpus | 0 of 23 | go |
| tsrs ≤ tsgo at `--checkers 4` on every target | 20–47 % of tsgo on all ten | go |
| startup of a one-file project ≤ 60 ms | 23 ms (range 22–24) | go |
| binary + `lib.*.d.ts` ≤ 40 MB | 32.5 MB (linux-x64-gnu; the largest package, linux-arm64-gnu, is 39.2 MB) | go |

**Go.** For `og dev` (D3, §4.5): a check of an example takes 34–41 ms at 4
checkers, beside a Bun rebuild in a separate process, so it runs in
parallel with the bundle and is reported on its own (`check.ms`), outside
the 300 ms rebuild + swap measure; Wave 2 re-measures the reloads with it
running (`evidence/wave-2.md`).
