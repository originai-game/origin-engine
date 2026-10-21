// og-game and og-ai write out the text formats an author types by hand
// (tilesets, maps, emitters, a map's and a model's import settings, the
// animation graph, the navmesh, behaviours) and what the cook reads of a
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

/** The skills with og-ai's text changed by `f`. */
function withAi(f: (s: string) => string): Map<string, string> {
  const m = skills();
  m.set("og-ai", f(m.get("og-ai")!));
  return m;
}

/** The sources with `path` changed by `f`. */
const withSource = (path: string, f: (s: string) => string) => (p: string) => (p === path ? f(read(p)!) : read(p));

test.skipIf(!inRepo)("the committed grammars match their parsers", () => {
  expect(grammarDrift(skills(), read)).toEqual([]);
});

test.skipIf(!inRepo)("each grammar's skill writes a block of it, and og-game the Tiled table", () => {
  const all = skills();
  for (const g of GRAMMARS) expect(all.get(g.home ?? "og-game")!).toContain(`type ${g.type}\n`);
  expect(all.get("og-game")!).toContain("<!-- grammar: tiled -->");
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

test.skipIf(!inRepo)("a navmesh key the cook does not read is caught, and one it reads that no block writes", () => {
  const p = grammarDrift(withAi((s) => s.replaceAll("  climb 0.4                    # the highest step", "  step 0.4                     # the highest step").replaceAll("  climb 0.3\n", "  step 0.3\n")), read);
  expect(p).toContain("crates/og-cook/src/cook/e6.rs: reads `climb`, which no `type navmesh` block in the skills writes or names; add it to og-ai's grammar");
  const added = grammarDrift(skills(), withSource("crates/og-cook/src/cook/e6.rs", (s) => s.replace('f32_of(agent, "slope", 45.0)', 'f32_of(agent, "maxSlope", 45.0)')));
  expect(added.some((x) => x.includes("reads `maxSlope`"))).toBeTrue();
  expect(added.some((x) => x.includes("writes `slope`") && x.includes("e6.rs"))).toBeTrue();
});

test.skipIf(!inRepo)("a behaviour node the parser does not know is caught", () => {
  const p = grammarDrift(withAi((s) => s.replace("      wait 0.3                 # runs this many seconds", "      pause 0.3                # runs this many seconds")), read);
  expect(p.some((x) => x.includes("writes `pause`") && x.includes("og-ai/src/asset.rs"))).toBeTrue();
});

test.skipIf(!inRepo)("a behaviour keyword, option or built-in key no block names is caught", () => {
  const node = grammarDrift(skills(), withSource("crates/og-ai/src/asset.rs", (s) => s.replace('"stop" => {', '"halt" | "stop" => {')));
  expect(node).toContain("crates/og-ai/src/asset.rs: reads `halt`, which no `type behaviour` block in the skills writes or names; add it to og-ai's grammar");
  const option = grammarDrift(skills(), withSource("crates/og-ai/src/asset.rs", (s) => s.replace('check_options(e, &["speed", "reach"])', 'check_options(e, &["speed", "reach", "lead"])')));
  expect(option.some((x) => x.includes("reads `lead`"))).toBeTrue();
  const key = grammarDrift(skills(), withSource("crates/og-ai/src/asset.rs", (s) => s.replace('"elapsed",\n];', '"elapsed",\n    "health",\n];')));
  expect(key.some((x) => x.includes("reads `health`"))).toBeTrue();
});

test.skipIf(!inRepo)("an animation graph's comparison the parser does not read is caught, both ways", () => {
  const p = grammarDrift(skills(), withSource("crates/og-anim/src/graph.rs", (s) => s.replace('"gt" => Op::Gt', '"greater" => Op::Gt')));
  expect(p.some((x) => x.includes("writes `gt`") && x.includes("og-anim/src/graph.rs"))).toBeTrue();
  expect(p.some((x) => x.includes("reads `greater`"))).toBeTrue();
});

test.skipIf(!inRepo)("a model's animation settings are held to the cook, and markers to their line", () => {
  const p = grammarDrift(skills(), withSource("crates/og-cook/src/cook/anim.rs", (s) => s.replace('"event" => a.events', '"cue" => a.events')));
  expect(p.some((x) => x.includes("writes `event`") && x.includes("og-cook/src/cook/anim.rs"))).toBeTrue();
  expect(p).toContain("crates/og-cook/src/cook/anim.rs: reads `cue`, which no `type import` animation block in the skills writes or names; add it to og-game's grammar");
});

test.skipIf(!inRepo)("the names in a behaviour, a graph and a navmesh are the author's", () => {
  // A state, a route, a label, a key, a command and its arguments; a motion, a parameter, a layer's state.
  const ai = grammarDrift(withAi((s) => s.replaceAll("state search", "state hunt").replaceAll("to search", "to hunt").replace("command bite 2 {", "command lunge far {").replace("sequence hurt {", "sequence wounded {")), read);
  expect(ai).toEqual([]);
  const graph = grammarDrift(withGame((s) => s.replaceAll("ground", "floor")), read);
  expect(graph).toEqual([]);
});

test.skipIf(!inRepo)("a grammar's region that moved is reported, not passed", () => {
  const p = grammarDrift(skills(), withSource("crates/og-cook/src/cook/e6.rs", (s) => s.replace("pub(super) fn cook_navmesh", "pub(super) fn bake_navmesh")));
  expect(p.some((x) => x.includes("e6.rs: the navmesh parser's region"))).toBeTrue();
});
