// Source maps: a game's stack back to its TypeScript (docs/web-export.md
// §Source maps). `og build` writes a map beside each bundle it makes
// (`game.js.map`, `play.js.map`; a publish build under `sourcemaps/`, which
// is never served), and this module is what reads them: the player shell
// maps a dev page's crash card with it, and the platform maps a stored crash
// report's `stack` with the release's stored maps
// (`@origingame/engine/sourcemap`):
//
//   import { mapStack } from "@origingame/engine/sourcemap";
//   mapStack(report.stack.join("\n"), { "game.js": gameMapJson, "play.js": playMapJson });
//
// Source map v3 without index sections (what Bun writes). No dependencies
// and no I/O: it runs in a browser, a Worker, bun and node alike.

/** A source map as JSON (v3). */
export interface RawSourceMap {
  readonly version: number;
  readonly sources: readonly (string | null)[];
  readonly names?: readonly string[];
  readonly mappings: string;
  readonly sourceRoot?: string;
  readonly sourcesContent?: readonly (string | null)[];
  readonly file?: string;
  readonly debugId?: string;
}

/** Where a bundle position came from: 1-based line and column, as a stack prints them. */
export interface OriginalPosition {
  readonly source: string;
  readonly line: number;
  readonly column: number;
  readonly name: string | null;
}

/** A decoded map: per generated line, its segments sorted by column. */
export class SourceMap {
  readonly sources: readonly string[];
  readonly names: readonly string[];
  readonly #mappings: string;
  /** Per generated line: [generated column, source, line, column, name] (0-based; -1 for none). */
  #lines: Int32Array[] | null = null;

  constructor(raw: RawSourceMap | string) {
    const map = (typeof raw === "string" ? JSON.parse(raw) : raw) as RawSourceMap;
    if (!map || map.version !== 3 || typeof map.mappings !== "string" || !Array.isArray(map.sources)) {
      throw new Error("not a version 3 source map (sections are not supported)");
    }
    const root = map.sourceRoot ? map.sourceRoot.replace(/\/?$/, "/") : "";
    this.sources = map.sources.map((s) => (s === null ? "" : root + s));
    this.names = map.names ?? [];
    this.#mappings = map.mappings;
  }

  /** The original position of a 1-based generated line and column, or null when the map says nothing for it. */
  original(line: number, column: number): OriginalPosition | null {
    const segments = this.#decoded()[line - 1];
    if (!segments || segments.length === 0) return null;
    const col = column - 1;
    // The last segment that starts at or before the column.
    let lo = 0;
    let hi = segments.length / 5 - 1;
    let at = -1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (segments[mid * 5]! <= col) {
        at = mid;
        lo = mid + 1;
      } else hi = mid - 1;
    }
    if (at < 0) return null;
    const source = segments[at * 5 + 1]!;
    if (source < 0) return null;
    const name = segments[at * 5 + 4]!;
    return { source: this.sources[source] ?? "", line: segments[at * 5 + 2]! + 1, column: segments[at * 5 + 3]! + 1, name: name >= 0 ? (this.names[name] ?? null) : null };
  }

  #decoded(): Int32Array[] {
    if (this.#lines) return this.#lines;
    const lines: Int32Array[] = [];
    const m = this.#mappings;
    let source = 0;
    let srcLine = 0;
    let srcCol = 0;
    let name = 0;
    let i = 0;
    let line: number[] = [];
    let genCol = 0;
    const vlq = (): number => {
      let result = 0;
      let shift = 0;
      for (;;) {
        const c = m.charCodeAt(i++);
        const digit = BASE64[c] ?? -1;
        if (digit < 0) throw new Error(`source map mappings: bad character at ${i - 1}`);
        result += (digit & 31) << shift;
        if ((digit & 32) === 0) break;
        shift += 5;
      }
      return result & 1 ? -(result >>> 1) : result >>> 1;
    };
    const endLine = () => {
      // Segments are sorted by column within a line in every map Bun writes; sort anyway.
      const n = line.length / 5;
      const order = Array.from({ length: n }, (_, k) => k).sort((a, b) => line[a * 5]! - line[b * 5]!);
      const out = new Int32Array(line.length);
      order.forEach((k, j) => out.set(line.slice(k * 5, k * 5 + 5), j * 5));
      lines.push(out);
      line = [];
      genCol = 0;
    };
    while (i < m.length) {
      const c = m.charCodeAt(i);
      if (c === 59 /* ; */) {
        i++;
        endLine();
        continue;
      }
      if (c === 44 /* , */) {
        i++;
        continue;
      }
      genCol += vlq();
      const segment = [genCol, -1, 0, 0, -1];
      if (i < m.length && m.charCodeAt(i) !== 44 && m.charCodeAt(i) !== 59) {
        source += vlq();
        srcLine += vlq();
        srcCol += vlq();
        segment[1] = source;
        segment[2] = srcLine;
        segment[3] = srcCol;
        if (i < m.length && m.charCodeAt(i) !== 44 && m.charCodeAt(i) !== 59) {
          name += vlq();
          segment[4] = name;
        }
      }
      line.push(...segment);
    }
    endLine();
    this.#lines = lines;
    return lines;
  }
}

const BASE64: Record<number, number> = {};
"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".split("").forEach((ch, k) => (BASE64[ch.charCodeAt(0)] = k));

/**
 * A stack frame's location, as V8 (`    at fn (url:1:2)`, `    at url:1:2`)
 * and JavaScriptCore and SpiderMonkey (`fn@url:1:2`) print it: the text
 * before it, the file, line and column, and the text after it.
 */
const FRAME = /^(.*?[\s(@]|)((?:[A-Za-z][\w+.-]*:\/{0,3})?[^\s()@]+?):(\d+):(\d+)(\)?\s*)$/;

/** A stack frame's file, line and column (1-based, as printed), or null for a line that is not a frame. */
export function frameAt(line: string): { readonly file: string; readonly line: number; readonly column: number } | null {
  if (!isFrame(line)) return null;
  const m = FRAME.exec(line)!;
  return { file: m[2]!, line: Number(m[3]), column: Number(m[4]) };
}

/** Whether a line of an error's text is a stack frame. */
export function isFrame(line: string): boolean {
  return /^\s*at\s/.test(line) ? FRAME.test(line) : /@/.test(line) && FRAME.test(line);
}

/** A location's file name: the last path segment of a URL or path, without its query or fragment. */
export function fileOf(location: string): string {
  const path = location.replace(/[?#].*$/, "");
  return path.slice(path.lastIndexOf("/") + 1);
}

/**
 * A file name without the content hash a publish build names it by
 * (`game.3f2a9c1e04b7.js` → `game.js`; docs/web-export.md §Content-hashed
 * names); any other name as it is.
 */
export function unhashed(file: string): string {
  return file.replace(/\.[0-9a-f]{12}(\.[A-Za-z0-9]+)$/, "$1");
}

/** What `mapStack` takes: maps by file name (`game.js`), or a function from a frame's location to its map. */
export type SourceMaps =
  | Readonly<Record<string, SourceMap | RawSourceMap | string | null | undefined>>
  | ((location: string) => SourceMap | RawSourceMap | string | null | undefined);

/**
 * A stack with each frame whose file has a map rewritten to the original
 * source, line and column (`at r (game.js:1:2345)` → `at r (src/main.ts:12:9)`);
 * every other line as it was. Maps given by name are looked up by the
 * frame's file name (a URL's last segment), so a report's frames match the
 * maps the build wrote whatever origin served it; a publish build's
 * content-hashed bundle (`game.3f2a9c1e04b7.js`) finds its map under the
 * bundle's own name (`game.js`).
 */
export function mapStack(stack: string, maps: SourceMaps): string {
  const cache = new Map<string, SourceMap | null>();
  const lookup = (location: string): SourceMap | null => {
    const key = typeof maps === "function" ? location : fileOf(location);
    if (cache.has(key)) return cache.get(key)!;
    let found: SourceMap | null = null;
    try {
      const raw = typeof maps === "function" ? maps(location) : (maps[key] ?? maps[unhashed(key)]);
      found = raw ? (raw instanceof SourceMap ? raw : new SourceMap(raw)) : null;
    } catch {
      found = null;
    }
    cache.set(key, found);
    return found;
  };
  return stack
    .split("\n")
    .map((line) => {
      if (!isFrame(line)) return line;
      const m = FRAME.exec(line)!;
      const map = lookup(m[2]!);
      const at = map?.original(Number(m[3]), Number(m[4]));
      return at ? `${m[1]}${at.source}:${at.line}:${at.column}${m[5]}` : line;
    })
    .join("\n");
}

/**
 * The stack frames of an error's text (its message may hold the stack of
 * the game's error inside it), each with its location cut to the file name
 * when it is a URL or an absolute path
 * (`at r (https://play.example/g/x/game.js:1:2345)` → `at r (game.js:1:2345)`),
 * at most `max` of them and 240 characters each: what a crash report
 * carries, with no origin, path, query or message text in it.
 */
export function stackFrames(text: string, max = 48): string[] {
  const out: string[] = [];
  for (const line of text.split("\n")) {
    if (out.length >= max) break;
    if (!isFrame(line)) continue;
    const m = FRAME.exec(line)!;
    // A URL or an absolute path names a machine or an origin: only its file name is kept.
    // A relative one (a mapped `src/main.ts`) is the project's own and stays.
    const file = /^([A-Za-z][\w+.-]*:|\/|\\)/.test(m[2]!) ? fileOf(m[2]!.replace(/\\/g, "/")) : m[2]!;
    out.push(`${m[1]!.trimStart()}${file}:${m[3]}:${m[4]}${m[5]!.trimEnd()}`.slice(0, 240));
  }
  return out;
}
