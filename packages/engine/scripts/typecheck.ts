// `bun run typecheck`: the package's three configurations through `og check`
// (the generated declarations with the examples and the
// contract, the runtime, the templates' sources), with the og the build
// put in native/ or $OG_BIN (tests/support/og-bin.ts). The CLI selects tsrs
// or its reported tsgo fallback on hosts without a tsrs package.
import { ogBin } from "../tests/support/og-bin.ts";
import { fileURLToPath } from "node:url";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; stdout: "inherit"; stderr: "inherit" }): { exitCode: number };
};

const pkg = fileURLToPath(new URL("..", import.meta.url));
let failed = 0;
for (const config of ["tsconfig.json", "tsconfig.runtime.json", "tsconfig.templates.json"]) {
  const r = Bun.spawnSync([ogBin(), "check", config], { cwd: pkg, stdout: "inherit", stderr: "inherit" });
  if (r.exitCode !== 0) failed++;
}
process.exit(failed === 0 ? 0 : 1);
