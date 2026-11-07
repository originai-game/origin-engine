// The build's dev check for state outside the world (docs/design.md §2.4):
// a game file's module-scope `let`/`var` that is written after its
// declaration, or a module-scope `const` container (`[]`, `{}`, `new Map`,
// `new Set`, ...) that is mutated, lives in the JS module, not in the world
// snapshot or the presentation checkpoint. A restore, a rollback or a capture
// fork does not bring it back, so the run after one differs from the run that
// was saved. Each finding names the file, line and variable; the runtime logs
// it as a warning in an engine's first log outside a publish build (src/runtime.ts, Engine.takeLog).
//
// A scan, not a type check: it reads the text with strings, template
// literals, comments and regular expressions skipped, and a local that
// shadows a module-scope name is reported as that name.

/** One module-scope variable the game writes. */
export interface ModuleState {
  readonly location: string;
  readonly name: string;
  readonly kind: "let" | "var" | "const";
}

const MUTATORS = "push|pop|shift|unshift|splice|sort|reverse|fill|copyWithin|set|add|delete|clear";
const ASSIGN = String.raw`(?:[-+*/%&|^]|\*\*|<<|>>>?|\?\?|&&|\|\|)?=(?![=>])`;

/** The module-scope state `source` (a game file, project-relative `file`) writes. */
export function moduleState(source: string, file: string): ModuleState[] {
  const code = blank(source);
  const found: ModuleState[] = [];
  const declared: { name: string; kind: ModuleState["kind"]; at: number; end: number }[] = [];
  // Statements at depth 0 that declare: `[export] let|var|const ...`.
  const top = /(^|[;}\n])\s*(?:export\s+)?(let|var|const)\s+/g;
  for (let m: RegExpExecArray | null; (m = top.exec(code)); ) {
    const at = m.index + m[0].length;
    if (depthAt(code, at) !== 0) continue;
    const kind = m[2] as ModuleState["kind"];
    const end = statementEnd(code, at);
    for (const d of declarators(code.slice(at, end))) {
      if (kind === "const" && !/^(?:\[|\{|new\s+(?:Map|Set|WeakMap|WeakSet|Array|Float32Array|Float64Array|Int32Array|Uint8Array|Uint16Array|Uint32Array|Int16Array|Int8Array)\b)/.test(d.init)) continue;
      declared.push({ name: d.name, kind, at, end });
    }
  }
  for (const d of declared) {
    const n = d.name.replace(/\$/g, "\\$");
    const rest = code.slice(0, d.at) + " ".repeat(d.end - d.at) + code.slice(d.end);
    const write = d.kind === "const"
      ? new RegExp(String.raw`(?<![\w$.])${n}(?:(?:\.[\w$]+|\[[^\]\n]*\])+\s*${ASSIGN}|\.(?:${MUTATORS})\s*\()`)
      : new RegExp(String.raw`(?<![\w$.])${n}\s*${ASSIGN}|(?:\+\+|--)\s*${n}(?![\w$])|(?<![\w$.])${n}\s*(?:\+\+|--)`);
    if (!write.test(rest)) continue;
    const line = source.slice(0, d.at).split("\n").length;
    found.push({ location: `${file}:${line}`, name: d.name, kind: d.kind });
  }
  return found;
}

/** The build's line appended to a game file with findings: the runtime reads it from `globalThis.__og_module_state`. */
export function moduleStateLine(found: readonly ModuleState[]): string {
  return found.length === 0 ? "" : `\n;(globalThis.__og_module_state ??= []).push(...${JSON.stringify(found)});\n`;
}

/** `source` with strings, template text, comments and regular expressions replaced by spaces (newlines kept, `${}` code kept). */
function blank(source: string): string {
  const out = source.split("");
  const wipe = (from: number, to: number) => {
    for (let k = from; k < to; k++) if (out[k] !== "\n") out[k] = " ";
  };
  // Template literal nesting: each entry is the brace depth its `${` opened at.
  const templates: number[] = [];
  let depth = 0;
  let last = ""; // the last significant character, for regex detection
  for (let i = 0; i < source.length; i++) {
    const c = source[i]!;
    const next = source[i + 1];
    if (c === "/" && next === "/") {
      const e = source.indexOf("\n", i);
      const to = e < 0 ? source.length : e;
      wipe(i, to);
      i = to - 1;
      continue;
    }
    if (c === "/" && next === "*") {
      const e = source.indexOf("*/", i + 2);
      const to = e < 0 ? source.length : e + 2;
      wipe(i, to);
      i = to - 1;
      continue;
    }
    if (c === '"' || c === "'") {
      let j = i + 1;
      for (; j < source.length && source[j] !== c && source[j] !== "\n"; j++) if (source[j] === "\\") j++;
      wipe(i + 1, j);
      i = j;
      last = "a";
      continue;
    }
    if (c === "`" || (c === "}" && templates.length > 0 && templates[templates.length - 1] === depth)) {
      if (c === "}") templates.pop();
      let j = i + 1;
      for (; j < source.length; j++) {
        if (source[j] === "\\") { j++; continue; }
        if (source[j] === "`") break;
        if (source[j] === "$" && source[j + 1] === "{") break;
      }
      wipe(i + 1, j);
      if (source[j] === "$") {
        templates.push(depth);
        i = j + 1; // past `${`
      } else i = j;
      last = "a";
      continue;
    }
    if (c === "/" && (last === "" || /[(,=:[!&|?{};+\-*%<>~^]/.test(last))) {
      let j = i + 1;
      let cls = false;
      for (; j < source.length && source[j] !== "\n"; j++) {
        if (source[j] === "\\") { j++; continue; }
        if (source[j] === "[") cls = true;
        else if (source[j] === "]") cls = false;
        else if (source[j] === "/" && !cls) break;
      }
      wipe(i + 1, j);
      i = j;
      last = "a";
      continue;
    }
    if (c === "{") depth++;
    else if (c === "}") depth--;
    if (!/\s/.test(c)) last = c;
  }
  return out.join("");
}

/** The bracket depth (`{`, `(`, `[`) at offset `at` of blanked code. */
function depthAt(code: string, at: number): number {
  let d = 0;
  for (let i = 0; i < at; i++) {
    const c = code[i];
    if (c === "{" || c === "(" || c === "[") d++;
    else if (c === "}" || c === ")" || c === "]") d--;
  }
  return d;
}

/** Where the declaration starting at `at` ends: the first `;` at its own depth, or a newline after a complete expression. */
function statementEnd(code: string, at: number): number {
  let d = 0;
  for (let i = at; i < code.length; i++) {
    const c = code[i]!;
    if (c === "{" || c === "(" || c === "[") d++;
    else if (c === "}" || c === ")" || c === "]") {
      if (d === 0) return i;
      d--;
    } else if (d === 0 && c === ";") return i;
    else if (d === 0 && c === "\n") {
      const before = code.slice(at, i).trimEnd();
      const after = code.slice(i + 1).trimStart();
      if (before !== "" && !/[,=+\-*/%&|^?:.<>!]$/.test(before) && !/^[.,?:+\-*/%&|^=<>)\]]/.test(after)) return i;
    }
  }
  return code.length;
}

/** The names a declaration list binds, each with its initializer's text (destructured names have none). */
function declarators(list: string): { name: string; init: string }[] {
  const parts: string[] = [];
  let d = 0;
  let start = 0;
  for (let i = 0; i < list.length; i++) {
    const c = list[i]!;
    if (c === "{" || c === "(" || c === "[" || c === "<") d++;
    else if (c === "}" || c === ")" || c === "]" || c === ">") d = Math.max(0, d - 1);
    else if (c === "," && d === 0) {
      parts.push(list.slice(start, i));
      start = i + 1;
    }
  }
  parts.push(list.slice(start));
  const out: { name: string; init: string }[] = [];
  for (const p of parts) {
    const t = p.trim();
    const simple = /^([A-Za-z_$][\w$]*)\s*(?:[:!][^=]*)?(?:=([\s\S]*))?$/.exec(t);
    if (simple) {
      out.push({ name: simple[1]!, init: (simple[2] ?? "").trim() });
      continue;
    }
    // `{ a, b: c } = ...` or `[a, b] = ...`: each bound identifier.
    const pattern = /^[{[]([\s\S]*?)[}\]]\s*(?::[^=]*)?=/.exec(t);
    if (!pattern) continue;
    for (const item of pattern[1]!.split(",")) {
      const name = /(?:^|:)\s*\.{0,3}([A-Za-z_$][\w$]*)\s*(?:=.*)?$/.exec(item.trim())?.[1];
      if (name) out.push({ name, init: "" });
    }
  }
  return out;
}
