// The probe wire's member tables state the same members once (audit R2):
// the shell's PROBE_WIRE_METHODS (packages/engine/shell/wire.ts) forward the
// engine's probe members, and og-probe's MEMBERS and PUBLISH_MEMBERS
// (crates/og-probe/src/probe.rs) are what a build serves. The tables live on
// both sides of the language boundary (the wire is the page's TypeScript, the
// probe is Rust), so this test pins them to each other: the wire's members
// are probe members, `entities` and `record` are the wire's own (state
// reshaped, a canvas recording), and the members a publish build refuses are
// exactly the dev ones the wire carries.
import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { PROBE_WIRE_METHODS } from "../shell/wire.ts";

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const probeSource = readFileSync(join(repo, "crates/og-probe/src/probe.rs"), "utf8");

/** The string literals of one `pub const …MEMBERS: &[&str]` table. */
function members(name: string): string[] {
  const block = new RegExp(`pub const ${name}: &\\[&str\\] = &\\[([\\s\\S]*?)\\];`).exec(probeSource);
  expect(block).not.toBeNull();
  return [...block![1]!.matchAll(/"([a-z_]+)"/g)].map((m) => m[1]!);
}

const rust = members("MEMBERS");
const publish = members("PUBLISH_MEMBERS");
const wire = [...PROBE_WIRE_METHODS];
/** The wire's own members, answered without the probe. */
const shellOnly = ["entities", "record"];

test("the wire's members are og-probe's, but its own two", () => {
  for (const method of wire) {
    if (shellOnly.includes(method)) continue;
    expect(rust).toContain(method);
  }
  expect(wire.filter((m) => !shellOnly.includes(m)).every((m) => rust.includes(m))).toBe(true);
});

test("a publish build refuses exactly the dev members the wire carries", () => {
  const devOnly = wire.filter((m) => rust.includes(m) && !publish.includes(m));
  expect(devOnly.sort()).toEqual(["advance", "bounds", "patch", "pause", "resume"]);
});
