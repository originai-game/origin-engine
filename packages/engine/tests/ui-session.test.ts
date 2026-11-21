import { expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawn(args: string[], options: { env: typeof process.env; stdout: "pipe"; stderr: "pipe" }): {
    stdout: ReadableStream<Uint8Array>; stderr: ReadableStream<Uint8Array>; exited: Promise<number>;
  };
  spawnSync(args: string[], options: { stdout: "pipe"; stderr: "pipe" }): unknown;
};

test.skipIf(process.platform === "win32")("UI checks share a host without sharing cells, strict exit status or failed setup", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-ui-session-"));
  const root = join(dir, "game");
  mkdirSync(root);
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "UI Session"\nentry "main.tsx"\nseed 83\n');
  // The menu's text passes og.i18n.t and the UI's own t(), which the
  // pseudo cell lengthens alike. A TSX menu; assets/ is the project's to cook.
  mkdirSync(join(root, "assets"));
  const source = `import {og} from "@origingame/engine";
import {Text,View,mount,t,useStore} from "@origingame/engine/ui";
let starts=0;
export default og.game({start(world){if(++starts!==1)throw new Error("cell state leaked");console.log(JSON.stringify({uiHost:process.pid}));og.ui.scope(world,"menu").writable("play","").set(og.i18n.t(world,"Play"));},ui(world){const play=useStore(og.ui.scope(world,"menu").writable("play",""));mount(world,()=><View class="flex-row gap-4"><View role="button" onPress={()=>{}}><Text>{play()}</Text></View><Text>{t("Quit")}</Text></View>,{});}});`;
  writeFileSync(join(root, "main.tsx"), source);
  const env = { ...process.env, OG_ENGINE_PACKAGE: resolve(import.meta.dir!, ".."), OG_RENDERER: "none", OG_SESSION: "1" };
  let hostPid = 0;
  let serial = 0;
  const invoke = async (cold = false, strict = false, failed = false) => {
    const out = join(dir, `out-${serial++}`);
    const child = Bun.spawn([ogBin(), "ui", "check", root, "--screen", "start=", "--breakpoints", "desktop", "--locales", "game,pseudo", "--scales", "100", "--settle", "0.05", "--out", out, "--json", ...(strict ? ["--require-checked"] : [])], {
      env: { ...env, OG_SESSION: cold ? "0" : "1" }, stdout: "pipe", stderr: "pipe",
    });
    const [code, stdout, stderr] = await Promise.all([child.exited, new Response(child.stdout).text(), new Response(child.stderr).text()]);
    expect(code, stdout + stderr).toBe(failed ? 2 : strict ? 1 : 0);
    if (failed) {
      expect(stderr).toContain("UI setup control");
      return null;
    }
    expect(stderr).not.toContain("running alone");
    const lines = stdout.trim().split("\n").map((line) => JSON.parse(line));
    const pids = lines.filter((line) => "uiHost" in line).map((line) => line.uiHost);
    expect(pids.length).toBe(2);
    expect(pids[0]).toBe(pids[1]);
    if (!cold) hostPid = pids[0];
    const { report, ...result } = lines.at(-1)!;
    expect(result.cells).toBe(2);
    expect(result.uncheckedCount).toBe(2);
    const frames = [0, 1].map((index) => JSON.parse(readFileSync(join(out, "cells", `${index}.uiframe.json`), "utf8")));
    expect(frames[0].elements.some((element: { text: string }) => element.text === "Play")).toBe(true);
    // The cells' texts, in the menu's order: Play's then Quit's.
    const shown = (index: number) => frames[index].elements.map((element: { text: string }) => element.text).filter(Boolean);
    expect(shown(0)).toEqual(["Play", "Quit"]);
    const [play, quit] = shown(1);
    expect(play).toStartWith("[Pláý ");
    expect(quit).toStartWith("[Qüít ");
    return { pid: pids[0], result, frames };
  };
  try {
    const cold = await invoke(true);
    const pair = await Promise.all([invoke(), invoke()]);
    expect(pair[0]?.pid).toBe(pair[1]?.pid);
    expect(pair[0]?.pid).not.toBe(cold?.pid);
    for (const warm of pair) {
      expect(warm?.result).toEqual(cold?.result);
      expect(warm?.frames).toEqual(cold?.frames);
    }
    expect((await invoke(false, true))?.pid).toBe(hostPid);
    writeFileSync(join(root, "main.tsx"), source.replace('if(++starts!==1)', 'throw new Error("UI setup control");if(++starts!==1)'));
    await invoke(false, false, true);
    writeFileSync(join(root, "main.tsx"), source);
    const recovered = await invoke();
    expect(recovered?.pid).toBe(pair[0]?.pid);
    expect(recovered?.frames).toEqual(cold?.frames);
  } finally {
    if (hostPid) Bun.spawnSync(["kill", "-TERM", String(hostPid)], { stdout: "pipe", stderr: "pipe" });
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);

test.skipIf(process.platform === "win32")("UI diagnostics report settled overflow, not historical layout warnings", async () => {
  const root = mkdtempSync(join(tmpdir(), "og-ui-warnings-"));
  try {
    mkdirSync(join(root, "assets"));
    writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "UI Warnings"\nentry "main.tsx"\nseed 83\n');
    // Transient starts as wide as Persistent and shrinks at 0.05 s; Persistent stays past the screen.
    writeFileSync(join(root, "main.tsx"), `import {og} from "@origingame/engine";
import {Text,View,mount,useStore} from "@origingame/engine/ui";
export default og.game({commands:{shrink(world){og.ui.scope(world,"menu").writable("wide",true).set(false);}},ui(world){const wide=useStore(og.ui.scope(world,"menu").writable("wide",true));mount(world,()=><View class="w-[1280px]"><View role="button" onPress={()=>{}} class={wide()?"w-[2000px] h-[30px]":"w-[100px] h-[30px]"}><Text>Transient</Text></View><View role="button" onPress={()=>{}} class="w-[2000px] h-[30px]"><Text>Persistent</Text></View></View>,{});}});`);
    const out = join(root, "out");
    const child = Bun.spawn([ogBin(), "ui", "check", root, "--screen", "start=0.05:shrink", "--breakpoints", "desktop", "--locales", "game", "--scales", "100", "--settle", "0.2", "--out", out, "--json"], {
      env: { ...process.env, OG_ENGINE_PACKAGE: resolve(import.meta.dir!, ".."), OG_RENDERER: "none", OG_SESSION: "0" }, stdout: "pipe", stderr: "pipe",
    });
    const [code, stdout, stderr] = await Promise.all([child.exited, new Response(child.stdout).text(), new Response(child.stderr).text()]);
    expect(code, stdout + stderr).toBe(0);
    // The button holding each label (its text's parent), and the safe-area
    // findings: what lies outside the screen once it settled.
    const frame = JSON.parse(readFileSync(join(out, "cells", "0.uiframe.json"), "utf8")) as { elements: { text: string; path: string }[] };
    const button = (label: string) => frame.elements.find((e) => e.text === label)!.path.split("/").slice(0, -2).join("/");
    const outside = (JSON.parse(readFileSync(join(out, "check.json"), "utf8")) as { findings: { rule: string; paths: string[] }[] }).findings
      .filter((f) => f.rule === "safe-area").flatMap((f) => f.paths);
    expect(outside).toContain(button("Persistent"));
    expect(outside).not.toContain(button("Transient"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}, 60_000);
