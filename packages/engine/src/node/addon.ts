import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/**
 * The addon the native loader uses: `OG_NATIVE_ADDON`, else this package's
 * `native/origin-engine.<platform>-<arch>.node` (a checkout's build), else the
 * per-platform package npm installed through optionalDependencies
 * (`@origingame/engine-<platform>-<arch>/origin-engine.node`).
 */
export function addonPath(): string {
  const fromEnv = process.env.OG_NATIVE_ADDON;
  if (fromEnv) return resolve(fromEnv);
  const here = dirname(fileURLToPath(import.meta.url));
  const platform = `${process.platform}-${process.arch}`;
  const local = join(here, "..", "..", "native", `origin-engine.${platform}.node`);
  if (existsSync(local)) return local;
  try {
    const require = createRequire(import.meta.url) as unknown as { resolve(id: string): string };
    return require.resolve(`@origingame/engine-${platform}/origin-engine.node`);
  } catch {
    return local;
  }
}
