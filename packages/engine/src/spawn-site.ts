// The TypeScript line that spawned an entity (SpawnDesc.source), outside a
// publish build: the first frame of world.spawn's stack that is the game's
// own source, project-relative (`src/enemies.ts:42`), as a UI element
// carries its document line. The probe answers it as the entity's source
// (pick, marks).
//
// A frame names the game's source when, made relative to the project, it
// stays inside it and is neither the engine (`@origingame/engine/…`) nor a
// package (node_modules). Bun maps a bundle's frames to their sources
// itself; a browser does not, so the web Worker hands over its bundle's map
// (`useSourceMap`). A frame no map covers and that is no source file (a
// bundle's own line) names nothing.

import { frameAt, SourceMap } from "./sourcemap.ts";

/** Frames read from a spawn's stack: the engine's own calls and a kit's few between it and the game. */
const FRAMES = 24;

/** Maps by the bundle URL or path their frames carry. */
const maps = new Map<string, SourceMap>();
/** A frame's location (with the project) to the site it names, or null for none. */
const sites = new Map<string, string | null>();
/** V8's and JavaScriptCore's frame cap (not in the ES lib). */
const errors = Error as ErrorConstructor & { stackTraceLimit: number };

/** Map `bundle`'s frames (its URL or path, as a stack prints it) to their sources with `map`. */
export function useSourceMap(bundle: string, map: SourceMap): void {
  maps.set(bundle, map);
  sites.clear();
}

/** The game's own line in the calling stack, or null when no frame is (the engine spawning, no map on the web). */
export function spawnSite(root: string | undefined): string | null {
  const limit = errors.stackTraceLimit;
  errors.stackTraceLimit = FRAMES;
  const stack = new Error().stack ?? "";
  errors.stackTraceLimit = limit;
  for (const line of stack.split("\n")) {
    const at = frameAt(line);
    if (!at) continue;
    const key = `${root ?? ""}\n${at.file}:${at.line}:${at.column}`;
    let site = sites.get(key);
    if (site === undefined) {
      site = siteOf(at.file, at.line, at.column, root);
      sites.set(key, site);
    }
    if (site !== null) return site;
  }
  return null;
}

function siteOf(file: string, line: number, column: number, root: string | undefined): string | null {
  const map = maps.get(file);
  if (map) {
    const at = map.original(line, column);
    const own = at && ownFile(at.source, root);
    return own ? `${own}:${at!.line}` : null;
  }
  // Unmapped: a source file bun mapped (an absolute path or file: URL), never a bundle's own line.
  if (!/\.[cm]?tsx?$/.test(file)) return null;
  const own = ownFile(file.replace(/^file:\/\//, ""), root);
  return own && `${own}:${line}`;
}

/** `path` relative to the project when it is the game's own source, else null. */
function ownFile(path: string, root: string | undefined): string | null {
  // macOS names one directory two ways (/var and /private/var): a frame bun maps through a bundle in the
  // temporary directory carries the /private form, the project root the other.
  const plain = (s: string) => s.replace(/\\/g, "/").replace(/^\/private(?=\/)/, "");
  let p = plain(path);
  const base = root === undefined ? undefined : plain(root).replace(/\/?$/, "/");
  if (base && p.startsWith(base)) p = p.slice(base.length);
  else if (/^([A-Za-z][\w+.-]*:|\/)/.test(p)) return null;
  if (p.startsWith("./")) p = p.slice(2);
  if (p === "" || p.startsWith("../") || p.startsWith("@origingame/") || /(^|\/)node_modules\//.test(p)) return null;
  return p;
}
