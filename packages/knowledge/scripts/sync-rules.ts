// Refreshes rules/platform.json, the platform knowledge rules this package is
// validated against, from an OriginGame platform checkout. It only reads the
// platform; the validator never does, so `bun run test` needs no checkout.
//
//   bun scripts/sync-rules.ts <origingame checkout> [--check]
//
// Each rule's `source` names the platform file it is transcribed from. The
// tool inventory is computed exactly as
// packages/game-knowledge/scripts/production.test.mjs computes it.

import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const here = join(import.meta.dir, "..");
const out = join(here, "rules/platform.json");
const args = process.argv.slice(2);
const platform = args.find((a) => !a.startsWith("--"));
if (!platform) {
  process.stderr.write("usage: bun scripts/sync-rules.ts <origingame checkout> [--check]\n");
  process.exit(2);
}
const knowledge = join(platform, "packages/game-knowledge");
const SNAKE = /^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$/;

const instructions = readFileSync(join(knowledge, "production/game/instructions.md"), "utf8");
const start = instructions.indexOf("## Tools");
const table = instructions.slice(start, instructions.indexOf("\n## ", start + 1));
const offered = new Set(
  table
    .split("\n")
    .filter((line) => line.startsWith("| ") && !line.startsWith("| Work |"))
    .flatMap((line) => [...line.split("|")[2]!.matchAll(/`([A-Za-z_]+)`/g)].map((m) => m[1]!)),
);
const guides = join(platform, "packages/origin-mcp/docs");
const documented = new Set<string>();
for (const file of readdirSync(guides).sort()) {
  for (const m of readFileSync(join(guides, file), "utf8").matchAll(/\b[a-z][a-z0-9]*(?:_[a-z0-9]+)+\b/g)) documented.add(m[0]);
}
const known = [...new Set([...offered, ...documented])].sort();
const skills = readdirSync(join(knowledge, "production/game/skills")).sort();
const overlays = readdirSync(join(knowledge, "production/game-scaffold")).sort();
const godotPin = JSON.parse(readFileSync(join(knowledge, "production/game-scaffold/origin-engine/origin-engine.json"), "utf8"));

const rules = {
  schemaVersion: 1,
  about: "The OriginGame platform's knowledge rules, transcribed as data so the package validates without a platform checkout. Refresh with `bun scripts/sync-rules.ts <origingame checkout>`.",
  payload: {
    roots: ["game", "game-scaffold"],
    encoding: "utf-8 (fatal decode, BOM kept)",
    regularFilesOnly: true,
    source: "packages/game-knowledge/scripts/production-bundle.mjs (readProductionBundle)",
  },
  packs: {
    skill: "game/skills/<name>/ -> pack skill:<name>",
    scaffold: "game-scaffold/<overlay>/ -> pack scaffold:<overlay>",
    instructions: "game/instructions.md -> pack instructions",
    other: "any other game/ path -> pack game",
    source: "agent-server/src/core/knowledge.ts (packsFromFiles)",
  },
  skills: {
    file: "SKILL.md",
    frontmatter: { required: ["name", "description"], nameIsDirectory: true },
    namePattern: "^[a-z0-9]+(?:-[a-z0-9]+)*$",
    existing: skills,
    source: "packages/game-knowledge/production/game/skills/*/SKILL.md (every skill's front matter: name equals its directory); agent-server/src/core/knowledge.ts (frontMatterDescription: the runner reads description)",
  },
  tools: {
    snake: SNAKE.source,
    notTools: ["text_server_adv"],
    retired: ["codemode"],
    known,
    source: "packages/game-knowledge/scripts/production.test.mjs (toolInventory and 'Game instructions and skills name only tools a Session can be offered'): production/game/instructions.md ## Tools plus packages/origin-mcp/docs/*.md",
  },
  scaffold: {
    pinFile: "origin-engine.json",
    pin: { schema: 1, probe: { protocol: godotPin.probe.protocol, wire: godotPin.probe.wire } },
    distributionFile: "distribution.json",
    distributionEntry: ["path", "bytes", "sha256", "licence", "source"],
    existingOverlays: overlays,
    source: "packages/game-knowledge/production/game-scaffold/origin-engine/{origin-engine.json,distribution.json}; scripts/production.test.mjs asserts pin.distribution and pin.probe.protocol 5; production/README.md: the payload is UTF-8 text only, binary files are named with digests in distribution.json",
  },
};
const text = JSON.stringify(rules, null, 2) + "\n";
if (args.includes("--check")) {
  const have = readFileSync(out, "utf8");
  if (have !== text) {
    process.stderr.write("rules/platform.json differs from the platform; run bun scripts/sync-rules.ts <checkout>\n");
    process.exit(1);
  }
  process.stdout.write("rules/platform.json matches the platform.\n");
} else {
  writeFileSync(out, text);
  process.stdout.write(`wrote rules/platform.json: ${known.length} known tools, ${skills.length} platform skills, overlays ${overlays.join(", ")}\n`);
}
