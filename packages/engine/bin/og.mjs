#!/usr/bin/env node
// `og` from an npm install: runs the og binary of this platform's package
// (@origingame/engine-<platform>-<arch>, an optionalDependency), with this
// package as the engine og runs games with. In a checkout, build og with
// `cargo build -p og-cli` instead.
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const platform = `${process.platform}-${process.arch}`;
let bin;
try {
  bin = join(dirname(createRequire(import.meta.url).resolve(`@origingame/engine-${platform}/package.json`)), process.platform === "win32" ? "og.exe" : "og");
} catch {
  process.stderr.write(`og: no @origingame/engine-${platform} package (this platform has no prebuilt og; the engine ships darwin-arm64 and linux-x64). Reinstall @origingame/engine with optional dependencies enabled.\n`);
  process.exit(1);
}
const engine = join(dirname(fileURLToPath(import.meta.url)), "..");
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit", env: { OG_ENGINE_PACKAGE: engine, ...process.env } });
if (r.error) {
  process.stderr.write(`og: cannot start ${bin} (${r.error.message})\n`);
  process.exit(1);
}
process.exit(r.status ?? 1);
