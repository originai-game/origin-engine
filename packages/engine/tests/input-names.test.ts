// The device, gamepad button and pad layout names, one source: the IDL's
// enums (crates/og-api/api/input.ogt) are what og-input's Device::NAMES
// (crates/og-input/src/glyph.rs), GAMEPAD_BUTTONS (src/event.rs) and
// PadLayout::NAMES (src/padmap.rs), and the web host's PAD_LAYOUTS
// (src/web/input.ts) repeat. This test pins them equal, in index order: the
// ring's layout index means nothing else.
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { PAD_LAYOUTS } from "../src/web/input.ts";

const here = dirname(fileURLToPath(import.meta.url));
const read = (path: string) => readFileSync(join(here, "..", "..", "..", path), "utf8");

const idl = read("crates/og-api/api/input.ogt");
const glyph = read("crates/og-input/src/glyph.rs");
const event = read("crates/og-input/src/event.rs");
const padmap = read("crates/og-input/src/padmap.rs");

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

test("the device, button and layout names are the IDL's, in the same order", () => {
  expect(rustStrings(glyph, "NAMES")).toEqual(idlVariants("InputDevice"));
  expect(rustStrings(event, "GAMEPAD_BUTTONS")).toEqual(idlVariants("GamepadButton"));
  const layouts = idlVariants("GamepadLayout");
  expect(Array.from<string>(PAD_LAYOUTS)).toEqual(layouts);
  expect(rustStrings(padmap, "NAMES")).toEqual(layouts);
});
