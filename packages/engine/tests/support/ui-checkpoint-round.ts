import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineEvent, defineResource, defineSystem, u32, type UiWritable, type RecordedIntent } from "@origingame/engine";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";

const host = process.argv[2];
const root = process.argv[3];
if (!root) throw new Error("expected cooked pause project");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else if (host === "web") {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(fileURLToPath(new URL("../../dist/web/origin-engine.js", import.meta.url)));
  await loadWebHost(glue, readFileSync(fileURLToPath(new URL("../../dist/web/origin-engine_bg.wasm", import.meta.url))));
  function preload(directory: string): void {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) preload(path);
      else if (entry.isFile()) glue.og_web_put_file(path, readFileSync(path));
    }
  }
  preload(join(root, "cooked"));
} else {
  throw new Error("expected node or web host");
}

const Intents = defineResource("CheckpointIntents", { confirms: u32() });
const Confirm = defineEvent("CheckpointConfirm", { amount: u32() });
let show = false;
let emit = false;
let hovered = false;
let focused = false;
let value: UiWritable<number>;
const Tick = defineSystem("CheckpointInput", { reads: [Confirm], writes: [Intents, Confirm, og.PauseIntent] }, (_q, _t, world) => {
  // Exercise the core control while the template parser is being replaced.
  // Simulation consumes the recorded input event, never the control response.
  if (show) {
    const response = og.ui.button("Confirm", "Confirm", { width: { kind: "fixed", px: 160 } });
    hovered ||= response.hovered;
    focused ||= response.focused;
    if (response.clicked && emit) {
      const confirmed = og.input.intent(world, Confirm, { amount: 3 });
      const paused = og.ui.push(world, "boundaryPause");
      if (!confirmed.ok || !paused.ok) throw new Error("intent refused");
    }
  }
  const intents = world.resource(Intents);
  if (!intents.ok) throw new Error(intents.reason);
  for (const event of world.events(Confirm)) {
    intents.value.confirms += event.amount;
  }
});

type Input = Parameters<Engine["probe"]["input"]>[0];
const tape: Input[][] = [
  [],
  [{ kind: "pointer", action: "move", x: 10, y: 10, button: "none" }],
  [{ kind: "key", action: "press", key: "Tab" }],
  [{ kind: "key", action: "press", key: "Enter" }],
  [],
  [],
];

function replay(presentation: boolean, confirm: boolean, recorded: readonly RecordedIntent[] = []) {
  show = presentation;
  emit = confirm;
  hovered = false;
  focused = false;
  const created = createEngine(og.game({
    seed: 73,
    resources: [Intents],
    systems: [Tick],
    start(world) {
      const document = og.load.document(world, "pause.svelte");
      if (!document.ok) throw new Error(document.reason);
      const sound = og.audio.createSound(world, { samples: new Float32Array([0.5, -0.5, 0.25, -0.25]) });
      if (!sound.ok) throw new Error(sound.reason);
      const configured = og.ui.configure(world, { sounds: { hover: sound.value, press: sound.value } });
      if (!configured.ok) throw new Error(configured.reason);
    },
    ui(world) { value = og.ui.scope(world, "checkpoint").writable("value", 7); },
  }) as unknown as GameToken, { root });
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  try {
    const before = engine.hash();
    if (presentation && !value.set(31).ok) throw new Error("store write refused");
    if (before !== engine.hash()) throw new Error("store write changed simulation hash");
    const hashes: string[] = [];
    const intentsRecorded: RecordedIntent[] = [];
    for (const [tick, inputs] of tape.entries()) {
      for (const input of inputs) {
        if (!engine.probe.input(input)) throw new Error("recorded input refused");
      }
      const advanced = engine.advance(1);
      if (!advanced.ok) throw new Error(advanced.reason);
      intentsRecorded.push(...og.input.takeIntents(engine.world));
      // Inputs emitted during a tick enter pending event queues at its end.
      // Replay them at that same boundary before comparing the pending hash.
      if (!presentation) {
        for (const intent of recorded.filter(intent => intent.tick === tick + 1)) {
          const beforeReplay = engine.hash();
          const refused = og.input.replayIntent(engine.world, { ...intent, layoutHash: (BigInt(intent.layoutHash) ^ 1n).toString() });
          if (refused.ok || !refused.reason.includes("layout hash")) throw new Error("layout mismatch not refused");
          if (engine.hash() !== beforeReplay) throw new Error("refused intent mutated world");
          const accepted = og.input.replayIntent(engine.world, intent);
          if (!accepted.ok) throw new Error(accepted.reason);
        }
      }
      hashes.push(engine.hash());
    }
    const intents = engine.world.resource(Intents);
    if (!intents.ok) throw new Error(intents.reason);
    const time = engine.world.resource(og.Time);
    if (!time.ok) throw new Error(time.reason);
    return { hashes, confirms: intents.value.confirms, scale: time.value.scale, hovered, focused, intentsRecorded };
  } finally { engine.destroy(); }
}

const withUi = replay(true, true);
console.log(JSON.stringify({ withUi, replay: replay(false, false, withUi.intentsRecorded), noIntent: replay(true, false), noUi: replay(false, false) }));
