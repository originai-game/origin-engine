// A game bundle's hash with the machine left out, the identity of a replay's
// game (`bundleSha256`, replay.ts): the same sources bundled in another
// checkout, by another user or on another machine hash the same.
//
// What the machine puts in a bundle, and what is dropped: the engine
// package's absolute path in its imports (written `@origingame/engine/`,
// Windows separators as `/`), bun's `// <path>` source markers, and its
// `//# debugId=` and `//# sourceMappingURL=` lines (the debug id is derived
// with the source map, whose paths are absolute).

import { readFileSync } from "node:fs";
import { dirname, resolve, sep } from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

/** The engine package this module belongs to. */
const PKG = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

/** The sha-256 of `file`, a bundle `src/node/build.ts` wrote, without its machine's paths. */
export function bundleHash(file: string, pkg: string = PKG): string {
  const text = readFileSync(file, "utf8")
    .split(JSON.stringify(pkg + sep).slice(1, -1))
    .map((part, i) => (i === 0 ? part : part.replace(/^[^"]*/, (rest) => rest.replaceAll("\\\\", "/"))))
    .join("@origingame/engine/")
    .split("\n")
    .filter((line) => !line.startsWith("// ") && !line.startsWith("//# "))
    .join("\n");
  return createHash("sha256").update(text).digest("hex");
}
