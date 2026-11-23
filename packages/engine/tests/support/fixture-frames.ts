// Every built-in component's fixtures laid out headless (no GPU): the
// native host's reading of the TSX tree (`og_node_ui_frame`), per fixture
// and per host, the way a game composes them (docs/ui.md §6.2). A fixture is
// mounted directly (mount() makes its root one grid cell, which
// stretches it to the surface) and as the one child of a game's flex column,
// where a screen sized to its content shows it.

import { og } from "@origingame/engine";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { bundleGame } from "../../src/node/build.ts";
import { cookProject } from "./og-bin.ts";
import { pages, type Page } from "../../src/node/gallery.ts";
import { loadNativeHost, native } from "../../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";
import { View, mount, renderers } from "../../src/ui/index.ts";

/** One element of a laid-out frame (og-ui-next's dump). */
export interface FrameElement {
  id: string;
  parent: string;
  path: string;
  role: string;
  text: string;
  name: string;
  disabled: boolean;
  clipped: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
  fontSize?: number;
  /** Each drawn line: [x, y, width, height], CSS px. */
  textLines?: number[][];
  /** What its ancestors' overflow leaves on screen: [x, y, width, height]. */
  clip?: number[];
  /** A press squashes it (the pack's `Press.feedback` or its own `active:` move). */
  press?: boolean;
  /** It enters from a `starting:` record and leaves to an `exit:` one. */
  presence?: Array<"enter" | "exit">;
  /** A `modal` view. */
  modal?: boolean;
  /** The fill its tint-mask frame (`nine-plate-cut`…) draws in: sRGB 0-255, alpha 0-1. */
  frame?: number[];
}

/** How a fixture is mounted: directly, or in a game's column; at a surface size. */
export interface Host {
  name: string;
  width: number;
  height: number;
  column: boolean;
  /** A touch screen: the pointer is a finger. */
  coarse: boolean;
}

export const HOSTS: Host[] = [
  { name: "phone", width: 390, height: 844, column: false, coarse: true },
  { name: "phone column", width: 390, height: 844, column: true, coarse: true },
  { name: "desktop", width: 1280, height: 720, column: false, coarse: false },
];

/** Bundles every built-in with fixtures once; then `frame(page, host)` lays one out. */
export async function fixtureFrames(filter?: (p: Page) => boolean) {
  // The built-ins only: an asset sheet is the gallery's preview of art, not a component a game mounts.
  const list = pages().filter((p) => !p.module.startsWith("sheets/") && (filter ?? (() => true))(p));
  const modules = [...new Set(list.map((p) => p.module))];
  const dir = mkdtempSync(join(tmpdir(), "og-fixture-frames-"));
  writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\nname \"fixtures\"\n");
  mkdirSync(join(dir, "src"));
  mkdirSync(join(dir, "assets"));
  const src = (m: string) => fileURLToPath(new URL(`../../src/${m}`, import.meta.url));
  // Cooked, so the components' string tables and the default pack's faces load: text measures as drawn.
  writeFileSync(join(dir, "src/main.ts"), modules.map((m, i) => `export * as M${i} from ${JSON.stringify(src(m))};`).join("\n") + "\n");
  cookProject(dir, dir);
  writeFileSync(join(dir, "main.ts"), modules.map((m, i) => `export * as M${i} from ${JSON.stringify(src(m))};`).join("\n") + "\n");
  const out = join(dir, "game.js");
  const b = await bundleGame(join(dir, "main.ts"), out, "native");
  if (!b.ok) throw new Error(b.reason);
  const rt = fileURLToPath(new URL("../../src/ui/rt.ts", import.meta.url));
  const ui = fileURLToPath(new URL("../../src/ui/index.ts", import.meta.url));
  writeFileSync(out, readFileSync(out, "utf8").replace(/(["'])[^"']*\/ui\/rt(\.ts)?\1/g, JSON.stringify(rt)).replace(/(["'])@origingame\/engine\/ui\1/g, JSON.stringify(ui)));
  const mods = (await import(out)) as Record<string, Record<string, unknown>>;
  loadNativeHost();

  // One engine per host, each fixture mounted and unmounted on it in turn.
  const engines = new Map<string, { engine: Engine; id: number }>();
  const engineFor = (host: Host) => {
    let e = engines.get(host.name);
    if (e) return e;
    const game = og.game({
      start(world) {
        og.ui.previewViewport(world, { width: host.width, height: host.height, dpr: 1, coarsePointer: host.coarse } as never);
        // The layout at rest: a fixture's entering rows (a feed sliding in) land at once.
        og.ui.configure(world, { reducedMotion: true });
      },
    });
    const created = createEngine(game as unknown as GameToken, { root: dir });
    if (!created.ok) throw new Error(created.reason);
    created.value.frame(1000 / 60);
    e = { engine: created.value, id: (created.value.world as unknown as { id: number }).id };
    engines.set(host.name, e);
    return e;
  };

  const frame = (page: Page, host: Host): FrameElement[] => {
    const mod = mods[`M${modules.indexOf(page.module)}`]!;
    const Component = mod[page.component] as (p: unknown) => never;
    const fixtures = mod.fixtures as Record<string, Record<string, unknown>>;
    const props = { ...fixtures.default, ...fixtures[page.fixture] };
    const { engine, id } = engineFor(host);
    const unmount = mount(engine.world, () => (host.column
      ? View({ role: "group", label: "column", class: "absolute inset-0 flex-col", children: Component(props) })
      : Component(props)), {});
    try {
      for (let i = 0; i < 3; i++) engine.frame(1000 / 60);
      return (JSON.parse(native().og_node_ui_frame(id)) as { elements: FrameElement[] }).elements;
    } finally {
      unmount();
      engine.frame(1000 / 60);
    }
  };
  const dispose = () => {
    for (const { engine, id } of engines.values()) { renderers.delete(id); engine.destroy(); }
    engines.clear();
    rmSync(dir, { recursive: true, force: true });
  };
  return { list, frame, dispose };
}
