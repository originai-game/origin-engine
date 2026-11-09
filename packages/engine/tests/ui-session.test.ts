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
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "UI Session"\nentry "main.ts"\nseed 83\n');
  // The menu's text passes og.i18n.t and the document's own t(), which the
  // pseudo cell lengthens alike.
  mkdirSync(join(root, "assets"));
  writeFileSync(join(root, "assets", "menu.svelte"), '<meta name="component" content="menu">\n<meta name="prop:play" content="string">\n<div id="Menu"><button id="Play">{$play}</button><span id="Quit">{t("Quit")}</span></div>\n');
  const source = `import {og} from "@origingame/engine";
let starts=0;
export default og.game({start(world){if(++starts!==1)throw new Error("cell state leaked");console.log(JSON.stringify({uiHost:process.pid}));const d=og.load.document(world,"menu.svelte");if(!d.ok)throw new Error(d.reason);og.ui.scope(world,"menu").writable("play","").set(og.i18n.t(world,"Play"));},ui(world){const s=og.ui.scope(world,"menu");const m=s.mount("menu","menu",{play:s.writable("play","")});if(!m.ok)throw new Error(m.reason);}});`;
  writeFileSync(join(root, "main.ts"), source);
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
    const shown = (index: number, id: string) => frames[index].elements.find((element: { id: string }) => element.id === id)?.text;
    expect([shown(0, "Play"), shown(0, "Quit")]).toEqual(["Play", "Quit"]);
    expect(shown(1, "Play")).toStartWith("[Pláý ");
    expect(shown(1, "Quit")).toStartWith("[Qüít ");
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
    writeFileSync(join(root, "main.ts"), source.replace('if(++starts!==1)', 'throw new Error("UI setup control");if(++starts!==1)'));
    await invoke(false, false, true);
    writeFileSync(join(root, "main.ts"), source);
    const recovered = await invoke();
    expect(recovered?.pid).toBe(pair[0]?.pid);
    expect(recovered?.frames).toEqual(cold?.frames);
  } finally {
    if (hostPid) Bun.spawnSync(["kill", "-TERM", String(hostPid)], { stdout: "pipe", stderr: "pipe" });
    rmSync(dir, { recursive: true, force: true });
  }
}, 60_000);
