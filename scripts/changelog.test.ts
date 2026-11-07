// scripts/changelog.ts: fragments fold into changelog.json's unreleased,
// and malformed ones are refused before anything is written.

import { expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { CHANGELOG, type Changelog, check, FRAGMENTS, fold, readLog } from "./changelog.ts";

function scratch(log: Changelog, fragments: Record<string, unknown>): string {
  const root = mkdtempSync(join(tmpdir(), "og-changelog-"));
  mkdirSync(join(root, "packages/engine"), { recursive: true });
  mkdirSync(join(root, FRAGMENTS), { recursive: true });
  writeFileSync(join(root, CHANGELOG), JSON.stringify(log));
  for (const [name, f] of Object.entries(fragments)) writeFileSync(join(root, FRAGMENTS, name), typeof f === "string" ? f : JSON.stringify(f));
  return root;
}

const base = (): Changelog => ({
  schema: 1,
  unreleased: { simulation: "unchanged", breaking: [], added: [{ member: "og.old", summary: "already folded" }], fixed: [] },
  versions: [{ version: "0.7.7", date: "2026-10-31", kind: "patch", simulation: "unchanged", breaking: [], added: [], fixed: ["x"] }],
});

test("fragments fold in name order, a changed simulation wins, and they are deleted", () => {
  const root = scratch(base(), {
    "b-motion.json": { simulation: "changed", breaking: [{ member: "og.motion.walkTo", change: "accelerates", fix: "set acceleration", find: ["walkTo("] }] },
    "a-ui.json": { added: [{ member: "og.ui.review", summary: "review sheets" }], fixed: ["a dropdown sliver"] },
    "c-vfx.json": { simulation: "unchanged", added: [{ member: "og.vfx", summary: "composed effects" }] },
  });
  expect(fold(root)).toEqual({ problems: [], folded: 3 });
  const u = readLog(root).unreleased;
  expect(u.simulation).toBe("changed");
  expect(u.added.map((n) => (typeof n === "string" ? n : n.member))).toEqual(["og.old", "og.ui.review", "og.vfx"]);
  expect(u.fixed).toEqual(["a dropdown sliver"]);
  expect(u.breaking.map((b) => b.member)).toEqual(["og.motion.walkTo"]);
  expect(readdirSync(join(root, FRAGMENTS))).toEqual([]);
  expect(readLog(root).versions).toEqual(base().versions);
});

test("a malformed fragment refuses the whole fold and writes nothing", () => {
  const root = scratch(base(), {
    "good.json": { added: [{ member: "og.a", summary: "fine" }] },
    "bad.json": { breaking: [{ member: "og.b", change: "no fix given" }] },
  });
  const { problems, folded } = fold(root);
  expect(folded).toBe(0);
  expect(problems.join("\n")).toContain('bad.json breaking[0]: "fix" is a non-empty string');
  expect(readLog(root)).toEqual(base());
  expect(readdirSync(join(root, FRAGMENTS)).sort()).toEqual(["bad.json", "good.json"]);
});

test("check names bad names, unknown keys, empty fragments and invalid JSON", () => {
  const root = scratch(base(), {
    "Upper.json": { added: ["x"] },
    "keys.json": { added: ["x"], notes: "?" },
    "empty.json": {},
    "broken.json": "{ not json",
  });
  const out = check(root).join("\n");
  expect(out).toContain("Upper.json: name it with lowercase letters");
  expect(out).toContain('keys.json: unknown key "notes"');
  expect(out).toContain("empty.json: records nothing");
  expect(out).toContain("broken.json: not JSON");
});

test("a patch with a breaking entry in changelog.json is a problem", () => {
  const log = base();
  log.versions[0]!.breaking.push({ member: "og.x", change: "c", fix: "f" });
  expect(check(scratch(log, {})).join("\n")).toContain("0.7.7: a patch has no breaking entries");
});

test("the repository's changelog and fragments are well formed", () => {
  expect(check()).toEqual([]);
});
