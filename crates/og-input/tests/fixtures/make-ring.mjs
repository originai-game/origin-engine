// Writes ring.bin and ring.json with the web shell's own ring code
// (packages/engine/src/web/input.ts), for tests/ring.rs:
//
//   node crates/og-input/tests/fixtures/make-ring.mjs [path/to/input.ts]
//
// ring.bin is the SharedArrayBuffer after a sequence of writes and drains
// that wraps the slot index; ring.json is what the TypeScript drainRing
// reads from it, plus the write and read indices.
import { writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const source = resolve(process.argv[2] ?? join(here, "../../../../packages/engine/src/web/input.ts"));
const { createRing, writeRing, drainRing, RING_SLOTS } = await import(source);

const ring = createRing();
// Fill and drain past the end so the live events straddle the wrap.
for (let i = 0; i < RING_SLOTS + 250; i++) {
  if (!writeRing(ring, { kind: "pointer", action: "move", x: i, y: -i, button: "none" })) throw new Error("full early");
  if (i % 2 === 1) drainRing(ring);
}
drainRing(ring);
const events = [
  { kind: "key", key: "KeyW", action: "down" },
  { kind: "pointer", action: "click", x: 12.5, y: 300.25, button: "left" },
  { kind: "wheel", x: 1, y: 2, deltaX: -3.5, deltaY: 120 },
  { kind: "key", key: "ArrowLeft", action: "press" },
  { kind: "pointer", action: "down", x: 0, y: 0, button: "right" },
  { kind: "pointer", action: "up", x: 640, y: 480, button: "middle" },
  { kind: "key", key: "NumpadSubtract", action: "up" },
  { kind: "pointer", action: "move", x: 0.1, y: 0.2, button: "none" },
  { kind: "pointer", action: "down", x: 900, y: 600, button: "left", id: 2 },
  { kind: "gamepad", pad: 1, connected: true, buttons: ["a", "leftBumper", "up"], axes: [0.5, -1, 0, 0.25], triggers: [0, 0.75] },
  { kind: "gamepad", pad: 1, connected: false, buttons: [], axes: [0, 0, 0, 0], triggers: [0, 0] },
  { kind: "text", text: "a" },
  { kind: "text", text: "にほんご", composing: true },
  // Past one slot's 48 bytes: split between characters, joined by the drain.
  { kind: "text", text: "Typed text longer than one slot holds: 日本語のテキスト" },
  // A screen reader's requests (a11y.ts): a path past 48 bytes is split as text is.
  { kind: "a11y", action: "activate", path: "Settings/Tabs/Audio" },
  { kind: "a11y", action: "increment", path: "Settings/Panel/Body/AudioPane/MasterRow/Master/Slider" },
];
for (const e of events) if (!writeRing(ring, e)) throw new Error("full");
const copy = new SharedArrayBuffer(ring.byteLength);
new Uint8Array(copy).set(new Uint8Array(ring));
const header = new Int32Array(ring);
const decoded = drainRing(copy);
writeFileSync(join(here, "ring.bin"), new Uint8Array(ring));
writeFileSync(
  join(here, "ring.json"),
  JSON.stringify({ write: header[0], read: header[1], slots: RING_SLOTS, bytes: ring.byteLength, events: decoded }, null, 2) + "\n",
);
console.log(`ring.bin ${ring.byteLength} bytes, write ${header[0]}, read ${header[1]}, ${decoded.length} events`);
