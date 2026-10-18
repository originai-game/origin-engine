// The native host's platform: og.platform's calls answered by the local
// platform (../platform-local.ts) from a JSON file, or from memory. `og run`
// keeps it in memory unless `--platform-store <file>` names a file, so a
// scripted run (a playtest) never depends on an earlier one.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { localPlatform, memoryPersistence, type LocalPersistence, type PlatformHost } from "../platform-local.ts";

/** The document in a file, created on the first save. */
export function filePersistence(path: string): LocalPersistence {
  return {
    load: () => (existsSync(path) ? readFileSync(path, "utf8") : null),
    save: (text) => {
      mkdirSync(dirname(path), { recursive: true });
      writeFileSync(path, text);
    },
  };
}

/** The native platform: a file's store when `path` is given, else memory. */
export function nativePlatform(path?: string): PlatformHost & { document(): unknown } {
  return localPlatform(path ? filePersistence(path) : memoryPersistence());
}
