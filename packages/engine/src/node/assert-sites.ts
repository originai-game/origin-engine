// The build's half of og.assert (src/assert.ts): each `og.assert.<kind>(`
// call in a game's own source gets its place (`src/player.ts:42`), read by
// the member from `this`, and the build lists every site with its name so a
// site that never runs is in the report too. Line numbers are kept: the
// rewrite never adds a line before a call.

type Kind = "always" | "sometimes" | "reachable" | "unreachable" | "each";

interface Declared {
  readonly kind: Kind;
  readonly name: string;
  readonly location: string;
}

/**
 * The build's rewrite of one game source file (src/node/build.ts): each
 * `og.assert.<kind>(` becomes `og.assert.<kind>.call("<file>:<line>", `,
 * and the sites are listed for the report. `file` is project-relative.
 */
export function placeAssertions(source: string, file: string): string {
  const sites: Declared[] = [];
  const nameAt: Record<Kind, number> = { always: 1, sometimes: 1, reachable: 0, unreachable: 0, each: 2 };
  const out = source.replace(/\bog\.assert\.(always|sometimes|reachable|unreachable|each)\s*\(/g, (match, kind: Kind, offset: number) => {
    const line = source.slice(0, offset).split("\n").length;
    const location = `${file}:${line}`;
    const name = literalArgument(source, offset + match.length, nameAt[kind]);
    if (name !== null) sites.push({ kind, name, location });
    return `og.assert.${kind}.call(${JSON.stringify(location)}, `;
  });
  if (out === source) return source;
  // `.call(loc, )` with no arguments (reachable()) is a syntax error in no engine; it is a call with none.
  const fixed = out.replace(/\.call\(("[^"\n]*"), \s*\)/g, ".call($1)");
  return sites.length === 0 ? fixed : `${fixed}\n;(globalThis.__og_assert_sites ??= []).push(...${JSON.stringify(sites)});\n`;
}

/** The `index`th argument of the call whose arguments start at `at`, when it is a plain string literal. */
function literalArgument(source: string, at: number, index: number): string | null {
  let depth = 0;
  let arg = 0;
  let start = at;
  for (let i = at; i < source.length; i++) {
    const c = source[i]!;
    if (c === '"' || c === "'" || c === "`") {
      const q = c;
      for (i++; i < source.length && source[i] !== q; i++) if (source[i] === "\\") i++;
      continue;
    }
    if (c === "(" || c === "[" || c === "{") depth++;
    else if (c === ")" || c === "]" || c === "}") {
      if (depth === 0) return arg === index ? literal(source.slice(start, i)) : null;
      depth--;
    } else if (c === "," && depth === 0) {
      if (arg === index) return literal(source.slice(start, i));
      arg++;
      start = i + 1;
    }
  }
  return null;
}

function literal(text: string): string | null {
  const t = text.trim();
  const m = /^(["'])((?:[^\\\n]|\\.)*?)\1$/.exec(t);
  if (!m) return null;
  try {
    return m[1] === '"' ? (JSON.parse(t) as string) : (JSON.parse(`"${m[2]!.replace(/"/g, '\\"')}"`) as string);
  } catch {
    return null;
  }
}
