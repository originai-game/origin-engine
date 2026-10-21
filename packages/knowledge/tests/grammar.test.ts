// og-game writes out the text formats an author types by hand (tilesets,
// maps, emitters, a map's import settings) and what the cook reads of a
// Tiled map. The validator holds each to its parser's source, both ways, so
// neither side can change without the other: these tests prove the
// committed grammar matches and that each kind of drift is caught.

import { expect, test } from "bun:test";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { PACKAGE_ROOT } from "../scripts/lib.ts";
import { grammarDrift, GRAMMARS, SKILLS, TILED } from "../scripts/validate.ts";

const repo = join(PACKAGE_ROOT, "..", "..");
const inRepo = existsSync(join(repo, "crates"));
const read = (p: string) => (existsSync(join(repo, p)) ? readFileSync(join(repo, p), "utf8") : undefined);
const skills = () => new Map(SKILLS.map((s) => [s, readFileSync(join(PACKAGE_ROOT, "skills", s, "SKILL.md"), "utf8")] as [string, string]));

/** The skills with og-game's text changed by `f`. */
function withGame(f: (s: string) => string): Map<string, string> {
  const m = skills();
  m.set("og-game", f(m.get("og-game")!));
  return m;
}

/** The sources with `path` changed by `f`. */
const withSource = (path: string, f: (s: string) => string) => (p: string) => (p === path ? f(read(p)!) : read(p));

test.skipIf(!inRepo)("the committed grammars match their parsers", () => {
  expect(grammarDrift(skills(), read)).toEqual([]);
});

test.skipIf(!inRepo)("og-game writes a block of every grammar and the Tiled table", () => {
  const game = skills().get("og-game")!;
  for (const g of GRAMMARS) expect(game).toContain(`type ${g.type}\n`);
  expect(game).toContain("<!-- grammar: tiled -->");
});

test.skipIf(!inRepo)("a key the parser does not read is caught", () => {
  const p = grammarDrift(withGame((s) => s.replace("  edge filled                 # filled", "  edges filled                # filled")), read);
  expect(p.some((x) => x.includes("writes `edges`") && x.includes("og-tilemap/src/tileset.rs"))).toBeTrue();
});

test.skipIf(!inRepo)("a word value the parser does not read is caught", () => {
  const p = grammarDrift(withGame((s) => s.replace("render sprite 12", "render sprites 12")), read);
  expect(p.some((x) => x.includes("writes `sprites`") && x.includes("og-cook/src/fx.rs"))).toBeTrue();
});

test.skipIf(!inRepo)("a key the parser reads and no block writes is caught", () => {
  const p = grammarDrift(skills(), withSource("crates/og-cook/src/fx.rs", (s) => s.replace('"glow" => em.glow', '"glow" => em.glow,\n            "trailLength" => em.glow')));
  expect(p).toContain("crates/og-cook/src/fx.rs: reads `trailLength`, which no `type particles` block in the skills writes or names; add it to og-game's grammar");
});

test.skipIf(!inRepo)("a renamed parser key is caught on both sides", () => {
  const p = grammarDrift(skills(), withSource("crates/og-tilemap/src/map.rs", (s) => s.replaceAll('"legend"', '"key"')));
  expect(p.some((x) => x.includes("writes `legend`"))).toBeTrue();
  expect(p.some((x) => x.includes("reads `key`"))).toBeTrue();
});

test.skipIf(!inRepo)("a map's import settings are held to the cook's tilemap block", () => {
  const p = grammarDrift(withGame((s) => s.replace('  level "Level_0"          # LDtk: the level, by identifier', '  levels "Level_0"         # LDtk: which one, by identifier')), read);
  expect(p.some((x) => x.includes("writes `levels`") && x.includes("og-cook/src/cook/tilemap.rs"))).toBeTrue();
  expect(p.some((x) => x.includes("reads `level`"))).toBeTrue();
});

test.skipIf(!inRepo)("the Tiled table is held to the importer, both ways", () => {
  const renamed = grammarDrift(withGame((s) => s.replace("| `autotile` |", "| `autoTile` |")), read);
  expect(renamed.some((x) => x.includes("names `autoTile`"))).toBeTrue();
  expect(renamed.some((x) => x.includes("reads `autotile`, which og-game's Tiled table does not name"))).toBeTrue();
  const added = grammarDrift(skills(), withSource(TILED.source, (s) => s.replace('.get("collision")', '.get("collision").or(props.get("solidLayer"))')));
  expect(added.some((x) => x.includes("reads `solidLayer`"))).toBeTrue();
});

test.skipIf(!inRepo)("a grammar dropped from the skills is caught", () => {
  const p = grammarDrift(withGame((s) => s.replaceAll("type particles\n", "type particle\n")), read);
  expect(p).toContain("skills: no skill writes a `type particles` block; og-game's grammar of it is gone");
});

test.skipIf(!inRepo)("names an author chooses are not grammar", () => {
  // `terrain wall`, `layer walls`, `object door`, `prop target`: free names.
  const p = grammarDrift(withGame((s) => s.replace("terrain wall {                # an auto-tiling", "terrain cliffs {              # an auto-tiling")), read);
  expect(p).toEqual([]);
});
