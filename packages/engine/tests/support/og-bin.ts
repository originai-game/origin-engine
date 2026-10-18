// The `og` binary a bun test runs (AGENTS.md §Build and gate discipline):
// $OG_BIN, else the one `scripts/build.sh` put in native/ for this platform.
// Tests never run `cargo`: under the gate build.sh has already run, and a
// standalone `bun test` says what to do when it has not.
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

export function ogBin(): string {
  const env = process.env.OG_BIN;
  if (env) return env;
  const platform = `${process.platform}-${process.arch}`;
  const bin = join(pkg, "native", `og.${platform}`);
  if (existsSync(bin)) return bin;
  throw new Error(`no og binary at ${bin}: run \`bun run build\` in packages/engine (or set OG_BIN)`);
}
