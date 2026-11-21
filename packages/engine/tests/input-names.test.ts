// The device and gamepad button names, one source: the IDL's enums
// (crates/og-api/api/input.ogt) are what og-input's Device::NAMES
// (crates/og-input/src/glyph.rs), og-input's GAMEPAD_BUTTONS
// (crates/og-input/src/event.rs) and the web host's PAD_DEVICES and
// GAMEPAD_BUTTONS (src/web/input.ts) repeat. This test pins them equal, in
// index order: the ring's bit mask and family index mean nothing else.
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { GAMEPAD_BUTTONS, PAD_DEVICES } from "../src/web/input.ts";

const here = dirname(fileURLToPath(import.meta.url));
const read = (path: string) => readFileSync(join(here, "..", "..", "..", path), "utf8");

const idl = read("crates/og-api/api/input.ogt");
const glyph = read("crates/og-input/src/glyph.rs");
const event = read("crates/og-input/src/event.rs");

/** The variants of the IDL enum named `name`, in order. */
function idlVariants(name: string): string[] {
  const body = idl.match(new RegExp(`enum ${name} \\{([\\s\\S]*?)\\n  \\}`))?.[1];
  if (!body) throw new Error(`no enum ${name} in input.ogt`);
  return [...body.matchAll(/^\s*variant (\w+)/gm)].map((m) => m[1]!);
}

/** The string literals of the `const ...: [...] =` array named `name`. */
function rustStrings(constDecl: string, name: string): string[] {
  const body = constDecl.match(new RegExp(`${name}\\s*:\\s*\\[[^=]*=\\s*\\[([\\s\\S]*?)\\]`))?.[1];
  if (!body) throw new Error(`no array ${name}`);
  return [...body.matchAll(/"([^"]+)"/g)].map((m) => m[1]!);
}

test("the device and button names are the IDL's, in the same order", () => {
  const devices = idlVariants("InputDevice");
  const buttons = idlVariants("GamepadButton");
  expect(Array.from<string>(PAD_DEVICES)).toEqual(devices);
  expect(rustStrings(glyph, "NAMES")).toEqual(devices);
  expect(Array.from<string>(GAMEPAD_BUTTONS)).toEqual(buttons);
  expect(rustStrings(event, "GAMEPAD_BUTTONS")).toEqual(buttons);
});
