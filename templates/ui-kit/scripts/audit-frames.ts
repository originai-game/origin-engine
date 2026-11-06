// bun templates/ui-kit/scripts/audit-frames.ts <base-ref>
// Checks the retained budget and immutable reference bytes; reports counts by owner.
import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { REFERENCE_FRAMES } from "../tests/reference-frames.ts";

const base = process.argv[2];
assert(base, "pass the pre-cut base commit");
const repo = resolve(import.meta.dir, "../../..");
const git = (...args: string[]) => execFileSync("git", args, { cwd: repo, maxBuffer: 32 * 1024 * 1024 });
const roots = ["templates/ui-kit/references", "templates/styles/references"];
const before = git("ls-tree", "-r", "--name-only", base, "--", ...roots).toString().trim().split("\n").filter(p => p.endsWith(".png"));
function pngs(dir: string): string[] {
  return readdirSync(join(repo, dir), { withFileTypes: true }).flatMap(e => e.isDirectory() ? pngs(join(dir, e.name)) : e.name.endsWith(".png") ? [join(dir, e.name)] : []);
}
const after = roots.flatMap(pngs);
const expected = Object.values(REFERENCE_FRAMES).flat().map(f => `templates/ui-kit/references/metal/${f}`);
for (const [kit, files] of Object.entries(REFERENCE_FRAMES)) assert(files.length <= 5, `${kit}: at most five reference frames`);
assert.equal(new Set(expected).size, expected.length, "each reference has one kit owner");
assert.deepEqual(after.filter(p => p.startsWith(roots[0]!)).sort(), expected.sort(), "kit reference files equal the explicit budget");
assert.deepEqual(after.filter(p => p.startsWith(roots[1]!)).sort(), before.filter(p => p.startsWith(roots[1]!)).sort(), "the already-small style sheets remain");
for (const path of after) {
  assert(before.includes(path), `no new frame: ${path}`);
  assert(readFileSync(join(repo, path)).equals(git("show", `${base}:${path}`)), `reference bytes unchanged: ${path}`);
}

// Mixed showcase sheets count once, under their primary family; all other
// old names identify the component or screen, independent of locale/scale.
function owner(path: string): string {
  const style = /^templates\/styles\/references\/([^/]+)\//.exec(path);
  if (style) return `style/${style[1]}`;
  const name = basename(path, ".png").replace(/^kit-/, "");
  const showcase = /^showcase-(.+?)(?:-phone)?$/.exec(name);
  if (showcase) {
    const families: Record<string, string> = {
      action: "ui-kit", shooter: "shooter", puzzle: "puzzle", management: "strategy", racing: "racing",
      moba: "rpg", rpg: "rpg", mmo: "rpg", roguelike: "card", strategy: "strategy", gacha: "gacha",
      rhythm: "rhythm", fighting: "versus", narrative: "story", survival: "rpg", horror: "ui-kit", openworld: "rpg",
    };
    assert(families[showcase[1]!], `known showcase: ${name}`);
    return families[showcase[1]!]!;
  }
  if (/^racing-/.test(name)) return "racing";
  if (/^starship-/.test(name)) return "strategy";
  if (/^(components|controls|locale)-/.test(name)) return "ui-kit";
  const screen = name.replace(/^(desktop|phone|patterns(?:-phone)?|genres[23]?(?:-phone)?|story(?:-phone)?)-/, "")
    .replace(/^(rtl|ja|ko|zh|text150)-/, "");
  for (const [kit, pattern] of [
    ["card", /^(cards|rewards|deck|rogueHud)/], ["gacha", /^gacha/], ["idle", /^generators/],
    ["liveops", /^(store|pass)/], ["photo", /^photo/], ["puzzle", /^(board|saga)/],
    ["racing", /^(race|garage)/], ["rhythm", /^(rhythmHud|songs)/],
    ["rpg", /^(gear|quests|dialogue|map-|worldMap|rpgHud|mmoHud|mobaHud|survivalHud|openHud)/],
    ["shooter", /^(shooter|loadout)/], ["sports", /^(sportsHud|matchFacts)/],
    ["story", /^(vn-|cg-|cgGallery|routes)/], ["strategy", /^(rts|ledger|citymap|cityMap|manageHud|tdHud)/],
    ["versus", /^(lobby|select|versus|draft|chat|fightHud|players)/], ["materials", /^materials/],
  ] satisfies Array<[string, RegExp]>) if (pattern.test(screen)) return kit;
  return "ui-kit";
}
const styles = readdirSync(join(repo, "templates/styles")).filter(name => existsSync(join(repo, "templates/styles", name, "extension.json")));
const owners = new Set([...Object.keys(REFERENCE_FRAMES), ...styles.map(name => `style/${name}`), ...before.map(owner)]);
console.log("| Owner | Before | After |\n|---|---:|---:|");
for (const kit of [...owners].sort()) {
  const old = before.filter(p => owner(p) === kit).length;
  const now = after.filter(p => owner(p) === kit).length;
  if (REFERENCE_FRAMES[kit]) assert.equal(now, REFERENCE_FRAMES[kit].length, `${kit}: classifier agrees with explicit owner`);
  console.log(`| ${kit} | ${old} | ${now} |`);
}
console.log(`| Total | ${before.length} | ${after.length} |`);
console.log(`\nAll ${after.length} retained PNGs are byte-identical to ${base}.`);
