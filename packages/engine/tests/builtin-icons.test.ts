// A built-in component draws with what the engine ships: every icon name a
// component's document writes must be in the default icon set
// (og_ui::DEFAULT_ICON_NAMES, the pinned vectors in third_party/lucide),
// so a game without an icon atlas never warns of a missing icon.
import { expect, test } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ENGINE = dirname(dirname(fileURLToPath(import.meta.url)));
const ROOT = join(ENGINE, "../..");
const DEFAULT = new Set(readdirSync(join(ROOT, "third_party/lucide/icons")).filter((f) => f.endsWith(".svg")).map((f) => f.slice(0, -4)));

function documents(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? documents(join(dir, e.name)) : e.name.endsWith(".svelte") ? [join(dir, e.name)] : [],
  );
}

/** The names a document draws: `icon="name"`, and each quoted name `icon={…}`
 * can yield (a branch of a conditional, not a value it is compared with). */
function named(text: string): string[] {
  const out: string[] = [];
  for (const m of text.matchAll(/\bicon="([^"{}]*)"/g)) if (m[1]) out.push(m[1]);
  for (const m of text.matchAll(/\bicon=\{([^}]*)\}/g))
    for (const q of m[1]!.matchAll(/(^|[?:(|&]\s*)["']([A-Za-z][\w-]*)["']/g)) out.push(q[2]!);
  return out;
}

/** The icon names a document's fixtures give its components (`"icon":"name"`,
 * `"costIcon":"name"`): the sheets draw them with no atlas too. */
function fixtureIcons(text: string): string[] {
  const out: string[] = [];
  for (const m of text.matchAll(/name="fixture:[^"]+" content=(?:"([^"]*)"|'([^']*)')/g)) {
    const json = (m[1] ?? m[2]!).replaceAll("&quot;", '"');
    for (const q of json.matchAll(/"(?:icon|[a-z]\w*Icon)"\s*:\s*"([A-Za-z][\w-]*)"/g)) out.push(q[1]!);
  }
  return out;
}

test("the default icon set is the one the engine pins", () => {
  // A regression that empties or moves the set would pass every name below.
  expect(DEFAULT.size).toBeGreaterThanOrEqual(20);
  expect(DEFAULT.has("check")).toBe(true);
});

test("every icon a built-in component names is in the default icon set", () => {
  const missing: string[] = [];
  for (const file of documents(join(ENGINE, "src/ui"))) {
    const text = readFileSync(file, "utf8");
    for (const name of [...named(text), ...fixtureIcons(text)]) if (!DEFAULT.has(name)) missing.push(`${relative(ENGINE, file)}: ${name}`);
  }
  // The screens sheet's own fixtures (sheets/screens/src/fixtures.ts).
  const sheet = readFileSync(join(ENGINE, "sheets/screens/src/fixtures.ts"), "utf8");
  for (const q of sheet.matchAll(/["'](?:icon|[a-z]\w*Icon)["']\s*:\s*["']([A-Za-z][\w-]*)["']/g)) if (!DEFAULT.has(q[1]!)) missing.push(`sheets/screens/src/fixtures.ts: ${q[1]}`);
  expect(missing).toEqual([]);
});
