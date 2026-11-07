import { expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { cookProject } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

test("cooked documents render typed spans and preserve game translations on both hosts", () => {
  const root = mkdtempSync(join(tmpdir(), "og-ui-documents-"));
  try {
    mkdirSync(join(root, "assets"));
    writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Documents"\nid "documents"\nentry "main.ts"\n');
    writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
    writeFileSync(join(root, "assets/card.svelte"), `<meta name="screen" content="card" data-layer="hud">
<meta name="strings" content="./base.en.ogt">
<column id="Root">
  <text id="Title">{t("card.title")}</text>
  <text id="Base">{t("card.base")}</text>
  <text id="Rich" spans={spans}>{body}</text>
  <text id="Plain">{body}</text>
  <panel id="Inherited" class={classes}/>
  <column id="Branch"><panel id="Nested"/></column>
  <panel id="Sibling"/>
</column>
<style>
#Root { width: 600px; --size: 32px; --saved: var(--size); --ramp: var(--text-xl3); }
#Rich::part(emphasis)[large=true] { font: var(--ramp); color: var(--danger); }
.chosen { --size: 80px; width: var(--saved) !important; }
#Inherited { width: 100px; height: 12px; }
#Branch { --size: 44px; }
#Nested, #Sibling { width: var(--size); height: 12px; }
</style>`);
    writeFileSync(join(root, "assets/base.en.ogt"), 'type strings\nversion 1\n---\nlocale "en"\nstring "card.title" "Base title"\nstring "card.base" "Base fallback"\n');
    writeFileSync(join(root, "assets/game.en.ogt"), 'type strings\nversion 1\n---\nlocale "en"\nstring "card.title" "Game title"\n');
    cookProject(root, root);
    const run = (host: string): unknown => {
      const result = Bun.spawnSync(["bun", fileURLToPath(new URL("./support/ui-documents-round.ts", import.meta.url)), host, root]);
      if (result.exitCode !== 0) throw new Error(new TextDecoder().decode(result.stderr));
      return JSON.parse(new TextDecoder().decode(result.stdout));
    };
    const node = run("node");
    expect(node).toMatchObject({ title: "Game title", base: "Base fallback", body: "A🦊B <b>C</b>", larger: true, restored: true, widths: [32, 44, 32] });
    expect(run("web")).toEqual(node);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
