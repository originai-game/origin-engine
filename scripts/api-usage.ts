// How often game code uses each member of the engine API, counted by the
// symbol the TypeScript checker resolves (not by name: `color` and `size`
// are fields of dozens of types). The input for the API's core/advanced
// tiers (`core` lines in crates/og-api/api/*.ogt) and the list of members
// no fixture exercises. A member nobody uses is not a removal candidate.
//
//   bun scripts/api-usage.ts              per module: items used, zero-use items
//   bun scripts/api-usage.ts --tsv        one row per entry (module, entry,
//                                         kind, game files, game sites, test files)
//   bun scripts/api-usage.ts --core       the tier rule applied: each module's
//                                         `core` line for crates/og-api/api
//
// Game code: tests/fixtures, templates (looks' sample sets), the project
// skeleton (packages/create/template), packages/engine/examples and the
// reference scene. Engine tests (packages/engine/tests) are a second column:
// coverage, not usage. Kits and style packs are CSS and markup, with no
// TypeScript to count.
import ts from "typescript@5.9.3";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = join(import.meta.dir, "..");
const engine = join(root, "packages/engine");
const dts = join(engine, "types/index.d.ts");

const GAME = [
  "tests/fixtures",
  "templates",
  "packages/create/template",
  "packages/engine/examples",
  "crates/og-tests/reference-scene",
];
const TESTS = ["packages/engine/tests"];

function files(dir: string, out: string[] = []): string[] {
  let names: string[];
  try {
    names = readdirSync(dir);
  } catch {
    return out;
  }
  for (const n of names) {
    if (n === "node_modules" || n === ".og" || n === "og-out" || n === "cooked") continue;
    const p = join(dir, n);
    const s = statSync(p, { throwIfNoEntry: false });
    if (s?.isDirectory()) files(p, out);
    else if (/\.tsx?$/.test(n) && !n.endsWith(".d.ts")) out.push(p);
  }
  return out;
}

const corpus = new Map<string, "game" | "test">();
for (const d of GAME) for (const f of files(join(root, d))) corpus.set(f, "game");
for (const d of TESTS) for (const f of files(join(root, d))) corpus.set(f, "test");

const program = ts.createProgram([...corpus.keys()], {
  target: ts.ScriptTarget.ES2022,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
  allowImportingTsExtensions: true,
  noEmit: true,
  strict: true,
  types: [],
  lib: ["lib.es2022.d.ts"],
  baseUrl: root,
  paths: {
    "@origingame/engine": [dts],
    "@origingame/engine/*": [join(engine, "src/*"), join(engine, "*")],
  },
});
const checker = program.getTypeChecker();
const api = program.getSourceFile(dts);
if (!api) throw new Error(`${dts} is not in the program`);

// ── The entries: every declaration of index.d.ts with an IDL name ──────────

type Entry = { name: string; kind: string; module: string; item: string };
const schema = JSON.parse(readFileSync(join(engine, "schema/api.schema.json"), "utf8"));
const moduleOf = new Map<string, string>();
for (const m of schema["x-og-modules"]) {
  for (const key of ["types", "interfaces", "callbacks", "values", "consts"]) {
    for (const x of m[key] ?? []) moduleOf.set(typeof x === "string" ? x : x.name, m.name);
  }
  for (const f of m.functions) {
    const name = typeof f === "string" ? f : f.name;
    const path = m.place === "top" ? name : m.place === "root" ? `og.${name}` : `og.${m.name}.${name}`;
    moduleOf.set(path, m.name);
  }
}
const entries = new Map<ts.Node, Entry>();
const byName = new Map<string, Entry>();
function add(node: ts.Node, name: string, kind: string, item: string, module?: string) {
  const top = item.replace(/^og\./, "");
  const e = byName.get(name) ?? {
    name,
    kind,
    module: module ?? moduleOf.get(item) ?? moduleOf.get(top) ?? "?",
    item,
  };
  byName.set(name, e);
  entries.set(node, e);
}
const records = new Set<string>();
function visitMembers(container: string, members: ts.NodeArray<ts.TypeElement>, item: string) {
  for (const m of members) {
    if (m.name && (ts.isIdentifier(m.name) || ts.isStringLiteral(m.name))) {
      add(m, `${container}.${m.name.text}`, ts.isMethodSignature(m) ? "method" : "field", item);
    }
  }
}
function visitOg(body: ts.ModuleBlock, path: string, module?: string) {
  for (const s of body.statements) {
    if (ts.isModuleDeclaration(s) && s.body && ts.isModuleBlock(s.body)) {
      visitOg(s.body, `${path}.${s.name.text}`, s.name.text);
    } else if (ts.isFunctionDeclaration(s) && s.name) {
      const name = `${path}.${s.name.text}`;
      add(s, name, "fn", name, module);
    } else if (ts.isVariableStatement(s)) {
      for (const d of s.declarationList.declarations) {
        if (!ts.isIdentifier(d.name)) continue;
        const name = `${path}.${d.name.text}`;
        const ty = d.type && ts.isTypeReferenceNode(d.type) ? d.type.typeName.getText(api) : "";
        const record = /^(Component|Resource|Event)Type$/.test(ty);
        if (record) records.add(d.name.text);
        add(d, name, record ? "record" : "value", name, module ?? moduleOf.get(d.name.text));
        // A record token's field specs: what `a.column(og.X).field` resolves to.
        const lit = d.type && ts.isTypeReferenceNode(d.type) ? d.type.typeArguments?.[0] : undefined;
        if (record && lit && ts.isTypeLiteralNode(lit)) visitMembers(name, lit.members, name);
      }
    }
  }
}
for (const s of api.statements) {
  if (ts.isModuleDeclaration(s) && s.name.text === "og" && s.body && ts.isModuleBlock(s.body)) {
    visitOg(s.body, "og");
  }
}
for (const s of api.statements) {
  if (ts.isInterfaceDeclaration(s)) {
    const n = s.name.text;
    // A record's value interface shares the token's entries.
    const name = records.has(n) ? `og.${n}` : n;
    if (!byName.has(name)) add(s, name, "type", name);
    else entries.set(s, byName.get(name)!);
    visitMembers(name, s.members, name);
  } else if (ts.isTypeAliasDeclaration(s)) {
    add(s, s.name.text, "type", s.name.text);
    if (ts.isTypeLiteralNode(s.type)) visitMembers(s.name.text, s.type.members, s.name.text);
  } else if (ts.isFunctionDeclaration(s) && s.name) {
    add(s, s.name.text, "fn", s.name.text);
  } else if (ts.isVariableStatement(s)) {
    for (const d of s.declarationList.declarations) {
      if (ts.isIdentifier(d.name)) add(d, d.name.text, "value", d.name.text);
    }
  }
}

// ── The references ─────────────────────────────────────────────────────────

const uses = new Map<Entry, { game: Set<string>; sites: number; test: Set<string> }>();
function entryOf(sym: ts.Symbol | undefined): Entry[] {
  if (!sym) return [];
  if (sym.flags & ts.SymbolFlags.Alias) sym = checker.getAliasedSymbol(sym);
  const out: Entry[] = [];
  for (const d of sym.declarations ?? []) {
    if (d.getSourceFile() !== api) continue;
    const e = entries.get(d);
    if (e) out.push(e);
  }
  return out;
}
function count(e: Entry, file: string, where: "game" | "test") {
  let u = uses.get(e);
  if (!u) uses.set(e, (u = { game: new Set(), sites: 0, test: new Set() }));
  if (where === "game") {
    u.game.add(file);
    u.sites++;
  } else u.test.add(file);
}
for (const [file, where] of corpus) {
  const sf = program.getSourceFile(file);
  if (!sf) continue;
  const rel = relative(root, file);
  const visit = (node: ts.Node) => {
    if (ts.isIdentifier(node)) {
      let syms: ts.Symbol[] = [];
      const p = node.parent;
      // A property written in an object literal: the property of the type
      // the literal is checked against (a desc's field), not the literal's own.
      if (
        (ts.isPropertyAssignment(p) || ts.isShorthandPropertyAssignment(p) || ts.isMethodDeclaration(p)) &&
        p.name === node &&
        ts.isObjectLiteralExpression(p.parent)
      ) {
        const t = checker.getContextualType(p.parent);
        const types = t ? (t.isUnion() ? t.types : [t]) : [];
        for (const ty of types) {
          const prop = checker.getPropertyOfType(checker.getApparentType(ty), node.text);
          if (prop) syms.push(prop);
        }
      } else {
        const s = checker.getSymbolAtLocation(node);
        if (s) syms.push(s);
      }
      const seen = new Set<Entry>();
      for (const s of syms) {
        for (const e of entryOf(s)) {
          if (seen.has(e)) continue;
          seen.add(e);
          count(e, rel, where);
          // A member's use is a use of what holds it.
          const owner = byName.get(e.item);
          if (owner && owner !== e && !seen.has(owner)) {
            seen.add(owner);
            count(owner, rel, where);
          }
        }
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(sf);
}

// ── Output ─────────────────────────────────────────────────────────────────

const all = [...byName.values()].filter((e) => e.module !== "?");
const used = (e: Entry) => uses.get(e)?.game.size ?? 0;
const items = all.filter((e) => e.name === e.item);
const args = process.argv.slice(2);
const gameFiles = [...corpus.values()].filter((w) => w === "game").length;
const testFiles = corpus.size - gameFiles;

if (args.includes("--tsv")) {
  console.log("module\tentry\tkind\tgame_files\tgame_sites\ttest_files");
  for (const e of all.sort((a, b) => a.module.localeCompare(b.module) || a.name.localeCompare(b.name))) {
    const u = uses.get(e);
    console.log(`${e.module}\t${e.name}\t${e.kind}\t${u?.game.size ?? 0}\t${u?.sites ?? 0}\t${u?.test.size ?? 0}`);
  }
} else if (args.includes("--core")) {
  // The tier rule: an item is core when at least one game file uses it.
  // Methods and fields count for the item that holds them. A module no game
  // file uses gets no core line, so all its items stay listed (and are
  // fixture coverage gaps). Prints the `core` lines for each module's IDL.
  const modules = [...new Set(items.map((e) => e.module))].sort();
  for (const m of modules) {
    const core = items
      .filter((e) => e.module === m && used(e) > 0)
      .map((e) => e.name.split(".").pop()!);
    const all = items.filter((e) => e.module === m).length;
    if (core.length > 0) console.log(`${m}\t${core.length}/${all}\tcore ${[...new Set(core)].join(" ")}`);
    else console.log(`${m}\t0/${all}\t(untiered: no game file uses it)`);
  }
} else {
  const modules = [...new Set(all.map((e) => e.module))].sort();
  console.log(`${gameFiles} game files, ${testFiles} engine test files\n`);
  console.log("module\titems used/all\tmembers used/all\tzero-use items");
  let iu = 0, it = 0, mu = 0, mt = 0;
  for (const m of modules) {
    const mi = items.filter((e) => e.module === m);
    const mm = all.filter((e) => e.module === m && e.name !== e.item);
    const zero = mi.filter((e) => used(e) === 0).map((e) => e.name.replace(/^og\./, ""));
    iu += mi.length - zero.length;
    it += mi.length;
    mu += mm.filter((e) => used(e) > 0).length;
    mt += mm.length;
    console.log(
      `${m}\t${mi.length - zero.length}/${mi.length}\t${mm.filter((e) => used(e) > 0).length}/${mm.length}\t${zero.slice(0, 12).join(", ")}${zero.length > 12 ? `, … ${zero.length - 12} more` : ""}`,
    );
  }
  console.log(`total\t${iu}/${it}\t${mu}/${mt}`);
  const untested = items.filter((e) => used(e) === 0 && (uses.get(e)?.test.size ?? 0) === 0);
  console.log(`\nNo game file and no engine test names these ${untested.length} items (fixture coverage gaps):`);
  console.log(untested.map((e) => `${e.module}: ${e.name}`).join("\n"));
}
