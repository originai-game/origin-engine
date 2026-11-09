// Each style pack's core-control state sheet (docs/ui.md §6.2): a pack with
// references under templates/styles/references/<pack>/ is rendered by
// `og ui sheet <pack>` as a game installs it, and every page's desktop
// (1920×1080) and phone (390×844) cell at 100 % text matches its reference
// (og-tests' tolerance, Metal on macrunner). OG_UI_CAPTURE=1 records them.
//
// Ways it could fail: a pack's art, fonts or icons no longer resolving once
// installed; a dressing that stops reaching its part (a renamed part, a
// selector the cascade drops); a sheet page that stops building its root.
import { expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { runOg } from "../../tests/support/og-bin.ts";
import { compareFrames, decodePng } from "../../tests/support/png.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const styles = join(pkg, "..", "..", "templates", "styles");
const capture = process.env.OG_UI_CAPTURE === "1";
const packs = readdirSync(join(styles, "references"), { withFileTypes: true })
  .filter((d) => d.isDirectory() && existsSync(join(styles, d.name, "extension.json")))
  .map((d) => d.name);

for (const pack of packs) {
  test(`${pack}: the core-control state sheet matches its references`, () => {
    const root = mkdtempSync(join(tmpdir(), `og-pack-sheet-${pack}-`));
    try {
      writeFileSync(join(root, "project.ogt"), `type project\nversion 1\n---\nname "sheet"\nid "sheet"\nentry "main.ts"\n`);
      writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
      const out = join(root, "out");
      const run = runOg(["ui", "sheet", pack, root, "--out", out], { OG_ENGINE_PACKAGE: pkg, OG_STYLES_DIR: styles });
      const said = run.stderr;
      if (run.status !== 0 && /GPU|adapter/i.test(said)) {
        console.warn(`WARNING ${pack} sheet unchecked: missing GPU: ${said.trim()}`);
        return;
      }
      expect(run.status, said).toBe(0);
      const index = JSON.parse(readFileSync(join(out, "index.json"), "utf8")) as {
        cells: Array<{ index: number; screen: string; breakpoint: string; scale: number }>;
      };
      const cells = index.cells.filter((c) => c.scale === 100);
      expect(cells.length).toBe(8);
      const moved: string[] = [];
      for (const c of cells) {
        const name = `sheet-${c.screen}-${c.breakpoint}.png`;
        const ref = join(styles, "references", pack, name);
        const got = readFileSync(join(out, "cells", `${c.index}.png`));
        if (capture) {
          mkdirSync(dirname(ref), { recursive: true });
          writeFileSync(ref, got);
          continue;
        }
        expect(existsSync(ref), `${ref}: record it with OG_UI_CAPTURE=1`).toBe(true);
        const d = compareFrames(decodePng(new Uint8Array(got)), decodePng(new Uint8Array(readFileSync(ref))));
        if (!d.ok) {
          moved.push(`${name}: RMSE ${d.rmse.toFixed(5)}, outliers ${(d.outliers * 100).toFixed(2)} %`);
          writeFileSync(join(tmpdir(), `og-pack-sheet-${pack}-${name}`), got);
        }
      }
      expect(moved, `moved frames (new captures in ${tmpdir()})`).toEqual([]);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  }, 240_000);
}
