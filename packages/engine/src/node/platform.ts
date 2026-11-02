// The native host's platform: og.platform's calls answered by the local
// platform (../platform-local.ts) from a JSON file, or from memory. `og run`
// keeps it in memory unless `--platform-store <file>` names a file, so a
// scripted run (a playtest) never depends on an earlier one. The platform
// services (purchases, ads, achievements, identity, share) are faked
// deterministically: by default every call succeeds, and `--platform-fake
// <file.json>` (a LocalFake; PlaytestDesc.platform) scripts their outcomes.

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { localPlatform, memoryPersistence, type LocalFake, type LocalPersistence, type PlatformHost } from "../platform-local.ts";

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

/** The native platform: a file's store when `path` is given, else memory; the services faked as `fake` scripts them. */
export function nativePlatform(path?: string, fake: LocalFake = {}): PlatformHost & { document(): unknown } {
  return localPlatform(path ? filePersistence(path) : memoryPersistence(), fake);
}

/** A LocalFake read from `--platform-fake <file.json>`, or the reason it cannot be. */
export function readFake(path: string): { ok: true; value: LocalFake } | { ok: false; reason: string } {
  try {
    const v = JSON.parse(readFileSync(path, "utf8")) as unknown;
    if (!v || typeof v !== "object" || Array.isArray(v)) return { ok: false, reason: `${path} is not a JSON object (a LocalFake: products, outcomes, player, friends, invites, notices, unavailable)` };
    return { ok: true, value: v as LocalFake };
  } catch (e) {
    return { ok: false, reason: `${path}: ${e instanceof Error ? e.message : String(e)}` };
  }
}
