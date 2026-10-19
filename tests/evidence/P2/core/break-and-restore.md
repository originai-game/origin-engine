# Core round: break and restore

Each new test was broken once by changing the code it covers (never the
test), seen to fail, and passed again once the change was reverted
(`packages/engine/tests/core.test.ts`: 7 of 7 pass after the restores;
`crates/og-audio/tests/mix.rs` and the archetype playtests pass).

## og.Rng in reads forks (core.test.ts)

Break: `packages/engine/src/runtime.ts` checks a fork as a write of og.Rng.

```diff
-      fork: (label: string) => (checkAccess("read", "og.Rng"), rngStream(abi().og_rng_fork(id, label))),
+      fork: (label: string) => (checkAccess("write", "og.Rng"), rngStream(abi().og_rng_fork(id, label))),
```

Failed: `expect(r.advanced.ok).toBeTrue()`, received `false` (the read-only
system's fork threw).

## An undeclared access throws in dev and release builds (core.test.ts)

Break: `checkAccess` lets every build refuse silently.

```diff
-  if (publishBuild === true) return false;
+  if (publishBuild !== null) return false;
```

Failed: `expect(a.ok).toBeFalse()`, received `true`.

## A publish build refuses and logs once per system (core.test.ts)

Break: `checkAccess` throws in the publish build too.

```diff
-  if (publishBuild === true) return false;
+  if (publishBuild === "x") return false;
```

Failed: `expect(a.ok).toBeTrue()`, received `false`.

## og.platform.now and the answers' time (core.test.ts)

Break: `Engine.answerPlatform` hands an untimed answer the time 0 instead of
the host clock's.

```diff
-    const timed = typeof answer.time === "number" ? { ...answer, clock: "player" } : { ...answer, time: this.#clock.now() };
+    const timed = typeof answer.time === "number" ? { ...answer, clock: "player" } : { ...answer, time: 0 };
```

Failed: `answerTime` expected 1767225600033.3333 (the fixed clock at tick 2),
received 0.

## og test keeps the store with store: keep (core.test.ts)

Break: `packages/engine/src/node/playtest-run.ts` never keeps the store.

```diff
-const store = desc!.store === "keep" ? join(out, "platform.json") : undefined;
+const store = false ? join(out, "platform.json") : undefined;
```

Failed: `FAIL away (7 checks, 1140 ms): a relaunch two hours later reads the
time away from the kept store`.

## Input events carry their time within the tick (core.test.ts)

Break: `crates/og-input/src/sink.rs` `InputLayer::inject_at` drops the stamp
(native addon rebuilt with `sh packages/engine/scripts/build.sh`).

```diff
-            at.clamp(-1.0, 1.0)
+            0.0 * at
```

Failed: `expect(node.seen).toEqual(...)`: every `at` 0 (0.25, -0.5, 0.75, 1,
0.498 and -0.5 expected).

## og.audio.outputLatency (core.test.ts)

Break: `packages/engine/src/web/audio.ts` `WorkerAudio.latency` leaves out
the ring's prime.

```diff
-    const seconds = context + AUDIO_PRIME_FRAMES / Math.max(1, this.#sampleRate);
+    const seconds = context + 0 * AUDIO_PRIME_FRAMES / Math.max(1, this.#sampleRate);
```

Failed: expected 0.054666… (0.012 s + 2048 / 48000), received 0.012.

## AudioSource.startOffset (crates/og-audio/tests/mix.rs)

Break: `crates/og-audio/src/system.rs` without the Cue command (the offset
never sent to the mixer).

Failed: `a_start_part_way_plays_the_clip_from_its_offset_to_the_sample`
"frame 0 of the part-way start is frame 15000 of the clip".

## idle: time away (templates/archetypes/idle/tests/away.og.ts)

Break: `src/ui.ts` with the time-away line disabled (`if (false) g.away = …`).

Failed: "away two hours by the hosts' clocks (0.000 s)", "the welcome back
is up: expected true, got false".

## rhythm: a press's time within the tick (templates/archetypes/rhythm/tests/timing.og.ts)

Break: `src/systems.ts` judges a press at its tick's start
(`const pressed = at;`, the press's `at` within the tick ignored).

Failed: "note 0's error is the press's 44 ms: expected 44 ± 0.1, got 50",
"note 0 at 44 ms is Perfect: expected 0, got 1", "note 2's error is the
press's 7 ms: expected 7 ± 0.1, got 0".

## rhythm: resume part-way into the stems (templates/archetypes/rhythm/tests/pause.og.ts)

Break: `src/systems.ts`'s Band starts every stem from its beginning
(`src.startOffset[i] = 0`).

Failed: "part-way into bar 2's stem: 0.650 s in, not from its start".
