// The art brief (scripts/brief.ts, docs/art.md §2.2): the expected sizes and
// parameters below are worked out by hand from the looks' numbers (written
// beside each), never read back from the code under test. The cases are
// asymmetric: a 2D pixel style and a 3D shape style ask for different things
// for the same kind, a character's cell is taller than wide, and the sizes
// snap to cells that are not powers of two.

import { afterAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { ART_KINDS, artBrief, audioBrief, presetLook, readAudioChoices, readLook } from "../scripts/brief.ts";
import { readProjectOutput } from "../scripts/project-output.ts";

const temps: string[] = [];
afterAll(() => {
  for (const t of temps) rmSync(t, { recursive: true, force: true });
});

const catalogue = JSON.parse(readFileSync(join(import.meta.dir, "..", "looks", "catalogue.json"), "utf8")) as { styles: { slug: string; look: { presetReady: boolean } }[] };

test("every kind for every preset makes a complete brief", () => {
  for (const s of catalogue.styles.filter((x) => x.look.presetReady)) {
    const look = presetLook(s.slug);
    for (const kind of ART_KINDS) {
      const b = artBrief(look, kind, "a weathered signpost");
      expect(b.style).toBe(s.slug);
      expect(b.prompt.split("\n")[0]).toContain("a weathered signpost");
      expect(b.prompt).toContain("Avoid: ");
      expect(b.references[0]?.role).toBe("style");
      expect((b.medium === "model") === (b.model !== undefined) && (b.medium === "image") === (b.image !== undefined)).toBe(true);
      if (b.image?.grid) expect([b.image.width, b.image.height]).toEqual([b.image.grid.width * b.image.grid.scale, b.image.grid.height * b.image.grid.scale]);
      if (b.model) expect(Math.log2(b.model.textureSize) % 1).toBe(0);
    }
  }
  expect(() => artBrief(presetLook("pixel"), "mesh" as never, "x")).toThrow(/asset kind/);
  expect(() => artBrief(presetLook("pixel"), "sprite", "  ")).toThrow(/subject/);
});

test("project stdout contains exactly one envelope beside launcher progress and propagates install errors", () => {
  const project = { startup: { look: "base/looks/t7", profile: "2d" }, input: { create: { music: "ambient" } } };
  const read = (stdout: string, status = 0, stderr = "") => readProjectOutput({ stdout, status, stderr });
  const native = JSON.stringify(project);
  expect(read(native)).toEqual(project);
  expect(read(`{"event":"resolve","version":"dev.20261127.1"}\r\n{"event":"installed","cached":false}\r\n${native}\r\n`)).toEqual(project);
  expect(read(`{"event":"installed","cached":true}\n${native}\n`)).toEqual(project);
  expect(() => read('{"event":"error","code":"signature","message":"bad signature","retryable":false}\n', 1, "generic exit")).toThrow("signature: bad signature");
  expect(() => read(`{"event":"error","code":"checksum","message":"mismatch"}\n${native}`)).toThrow("checksum: mismatch");
  expect(() => read(`{"event":"installed"}\n`)).toThrow("no project object");
  expect(() => read(`${native}\n${native}`)).toThrow("received multiple");
  expect(() => read("", 2, "invalid project")).toThrow("invalid project");
  expect(() => read("not JSON")).toThrow();
  expect(() => read("null")).toThrow("expected a JSON object");
});

test.skipIf(!process.env.OG_BIN)("project briefs use the restricted JSON/TS bootstrap and refresh imported choices", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-brief-ts-"));
  temps.push(dir);
  mkdirSync(join(dir, "src"));
  mkdirSync(join(dir, "lib"));
  mkdirSync(join(dir, "content/looks"), { recursive: true });
  mkdirSync(join(dir, "assets/palettes"), { recursive: true });
  writeFileSync(join(dir, "og-project.json"), JSON.stringify({ version: 1, engine: "0.9.0", name: "Brief", look: { $ogRef: "base/looks/brief" } }));
  writeFileSync(join(dir, "project.ts"), `import { defineProject } from "@origingame/engine/author";
import { choices } from "./lib/choices.ts";
export default defineProject({ id: "brief", entry: "src/main.ts", create: choices });`);
  writeFileSync(join(dir, "src/main.ts"), "throw new Error('GAME_MUST_NOT_EXECUTE');");
  writeFileSync(join(dir, "lib/grade.ts"), 'export const mood = "cold";');
  writeFileSync(join(dir, "content/looks/base.ts"), 'import {file,ref} from "@origingame/engine/author"; export default file.look({extends:["pixel"],ArtPalette:{colors:ref("base/palettes/brief")}});');
  writeFileSync(join(dir, "content/looks/brief.ts"), 'import {file,ref} from "@origingame/engine/author"; import {mood} from "../../lib/grade.ts"; export default file.look({extends:[ref("base/looks/base")],mood});');
  writeFileSync(join(dir, "assets/palettes/brief.hex"), "#123456\n#abcdef\n");
  const choices = join(dir, "lib/choices.ts");
  writeFileSync(choices, 'export const choices = { form: "2d", music: "ambient", genre: "rpg" };');
  const cook = () => {
    const result = Bun.spawnSync([process.env.OG_BIN!, "cook", dir, "--json"]);
    expect(result.exitCode, new TextDecoder().decode(result.stderr)).toBe(0);
  };
  expect(() => readLook(dir)).toThrow("run og cook");
  cook();
  const look = readLook(dir);
  expect([look.profile, look.style?.slug, look.mood]).toEqual(["2d", "pixel", "cold"]);
  expect(look.palette?.colours.map(c => c.hex)).toEqual(["#123456", "#abcdef"]);
  expect(readAudioChoices(dir)).toEqual({ music: "ambient", genre: "rpg", mood: "cold" });
  const audio = audioBrief(readAudioChoices(dir));
  expect(audio.presets.length).toBeGreaterThan(0);
  for (const preset of audio.presets) {
    expect(preset.file).toBe(`content/audio/${preset.id}.ts`);
    mkdirSync(join(dir, "content/audio"), { recursive: true });
    writeFileSync(join(dir, preset.file), preset.ts);
  }
  const cooked = Bun.spawnSync([process.env.OG_BIN!, "cook", dir, "--json"]);
  expect(cooked.exitCode, new TextDecoder().decode(cooked.stderr)).toBe(0);
  const indexed = Bun.spawnSync([process.env.OG_BIN!, "assets", "--project", dir, "--json"]);
  expect(indexed.exitCode).toBe(0);
  const assets = JSON.parse(new TextDecoder().decode(indexed.stdout)).assets as { id: string; kind: string }[];
  for (const preset of audio.presets) {
    expect(assets.find((a) => a.id === `base/audio/${preset.id}`)?.kind).toBe("sound");
  }
  // Entry, helper, transitive look and raw palette edits must all invalidate
  // a previously successful report, even if the chosen startup ID is unchanged.
  for (const file of ["content/looks/brief.ts", "lib/grade.ts", "content/looks/base.ts", "assets/palettes/brief.hex"]) {
    const path = join(dir, file);
    const original = readFileSync(path, "utf8");
    writeFileSync(path, original + (file.endsWith(".ts") ? "\n// edited\n" : "#fedcba\n"));
    expect(() => readLook(dir)).toThrow("run og cook");
    cook();
    expect(readLook(dir).mood).toBe("cold");
    rmSync(path);
    expect(() => readLook(dir)).toThrow("run og cook");
    writeFileSync(path, original);
    cook();
  }
  writeFileSync(choices, 'export const choices = { form: "3d", music: "orchestral", genre: "strategy" };');
  expect(() => readLook(dir)).toThrow("run og cook");
  cook();
  expect(readLook(dir).profile).toBe("3d");
  expect(readAudioChoices(dir)).toEqual({ music: "orchestral", genre: "strategy", mood: "cold" });
  writeFileSync(choices, 'export const choices = { form: "3d", music: String(Date.now()), genre: "strategy" };');
  expect(() => readLook(dir)).toThrow();
}, 120_000);
