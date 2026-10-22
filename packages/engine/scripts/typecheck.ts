// `bun run typecheck`: the package's three configurations through `og check
// --tool tsrs` (the generated declarations with the examples and the
// contract, the runtime, the templates' sources), with the og the build
// put in native/ or $OG_BIN (tests/support/og-bin.ts). tsrs is the checker;
// tsgo is only check-conformance's comparison (scripts/tsrs-diff.sh).
import { ogBin } from "../tests/support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; stdout: "inherit"; stderr: "inherit" }): { exitCode: number };
};

const pkg = new URL("..", import.meta.url).pathname;
let failed = 0;
for (const config of ["tsconfig.json", "tsconfig.runtime.json", "tsconfig.templates.json"]) {
  const r = Bun.spawnSync([ogBin(), "check", config, "--tool", "tsrs"], { cwd: pkg, stdout: "inherit", stderr: "inherit" });
  if (r.exitCode !== 0) failed++;
}
process.exit(failed === 0 ? 0 : 1);
