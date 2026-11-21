// `bun run typecheck`: the package's three configurations through `og check`
// (the generated declarations with the examples and the
// contract, the runtime, the templates' sources), with the og the build
// put in native/ or $OG_BIN (tests/support/og-bin.ts). The CLI selects tsrs
// or its reported tsgo fallback on hosts without a tsrs package. Then the
// built-in components' colours: a class names a theme role (`bg-hud-accent/24`),
// never a literal (`bg-[#38e8ff3d]`), so a style pack restyles every one.
import { ogBin } from "../tests/support/og-bin.ts";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
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
const ui = join(pkg, "src/ui");
const literal = /(?<![\w-])(?:[\w-]+:)*[a-z]+(?:-[a-z]+)*-\[#[0-9a-fA-F]{3,8}\]/g;
const walk = (dir: string): void => {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const file = join(dir, e.name);
    if (e.isDirectory()) walk(file);
    else if (e.name.endsWith(".tsx")) {
      readFileSync(file, "utf8").split("\n").forEach((line, i) => {
        for (const m of line.matchAll(literal)) {
          console.error(`${file.slice(pkg.length)}:${i + 1}: \`${m[0]}\` is a literal colour: name its theme role (packages/design/tokens/color.css)`);
          failed++;
        }
      });
    }
  }
};
walk(ui);
process.exit(failed === 0 ? 0 : 1);
