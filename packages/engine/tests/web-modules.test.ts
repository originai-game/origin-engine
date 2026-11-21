// A publish build of a split engine (docs/web-export.md §Modules): it
// records the modules the game references, which the boot fetches beside
// the primary, and serves the rest on their first call. An engine whose
// modules no CDN serves (built in a checkout), and `--modules local`, copy
// every module beside the page.
import { expect, test } from "bun:test";
import { cpSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { settingsOf, webBuild } from "../src/node/web-build.ts";
import { cookProject } from "./support/og-bin.ts";
import { slow } from "./support/slow.ts";

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const NO_SPLIT = !existsSync(join(pkg, "dist", "web-publish", "modules", "modules.json"));

test.skipIf(NO_SPLIT)("a publish build references the modules its game names and ships them beside the page when no CDN serves them", async () => {
  const work = mkdtempSync(join(tmpdir(), "og-modules-"));
  try {
    const project = join(work, "game");
    cpSync(join(repo, "tests", "fixtures", "browser-host"), project, { recursive: true });
    for (const d of ["cooked", "dist", "og-out", ".og"]) rmSync(join(project, d), { recursive: true, force: true });
    // A game naming physics and a dialogue: those two (with its 3D profile's simulation) are referenced, nothing else.
    writeFileSync(join(project, "src", "uses.ts"), "import { og } from \"@origingame/engine\";\nexport const used = [og.RigidBody, og.story.start];\n");
    cookProject(project, repo);
    const out = join(work, "dist");
    const b = await webBuild(project, join(project, "src", "main.ts"), out, true, { ...settingsOf(project), publish: { ...settingsOf(project).publish, modules: "local" } });
    if (!b.ok) throw new Error(b.reason);
    const m = b.manifest!.modules!;
    expect(m.referenced).toEqual(["physics", "physics3d", "story"]);
    expect(m).toMatchObject({ base: "modules/", origin: null, local: true });
    // Every module is beside the page, so an unreferenced one still loads on its first call.
    const listed = JSON.parse(readFileSync(join(pkg, "dist", "web-publish", "modules", "modules.json"), "utf8")) as Record<string, { file: string }>;
    for (const e of Object.values(listed)) expect(existsSync(join(out, e.file))).toBe(true);
    // The page hands the Worker the same list.
    const config = JSON.parse(/<script type="application\/json" id="og-config">([^<]*)<\/script>/.exec(readFileSync(join(out, "index.html"), "utf8"))![1]!);
    expect(config.modules.single.referenced).toEqual(["physics", "physics3d", "story"]);
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}, slow(180_000));
