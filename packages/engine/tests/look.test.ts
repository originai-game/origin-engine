// Looks through the TypeScript API on the native host (IDL `load.look`,
// `load.lookInfo`, `load.palette`, `render.applyLook`, `render.look`,
// `GameDesc.look`; og-cook's `look` module): a cooked look
// (tests/fixtures/look) is flattened over the look it extends, applied
// before GameDesc.start runs, and sets the resources it names whole (the
// fields it names from the look, the rest at their defaults, the animated
// ones kept) while leaving the others alone; its skin configures the UI and
// its antialias is the renderer's; refusals name the fix.
import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const fixture = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "look");

// At module level, not in a hook: a hook's 5 s would not cover a cook on a
// loaded machine.
{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", fixture]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(fixture, "cooked"))) throw new Error(`og cook ${fixture} wrote no cooked/`);
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

type Info = { look: number; path: string; name: string; extends: string[]; resources: string[]; skin: number | null; antialias: string | null };
type Res = Record<string, unknown> & { [k: string]: any };

function start(desc: Record<string, unknown>, seen?: (w: World) => void): Engine {
  const game = og.game({ ...desc, start: seen ?? (() => {}) } as never);
  const r = createEngine(game as GameToken, { root: fixture });
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const res = (w: World, token: unknown): Res => must(w.resource(token as never)) as Res;

test("GameDesc.look is flattened over its base and applied before start runs", () => {
  let atStart: { stylise: Res; settings: Res; fog: Res; post: Res; style: Res; look: Info | null } | undefined;
  const engine = start({ look: "looks/dusk.ogt" }, (w) => {
    atStart = {
      stylise: res(w, og.Stylise),
      settings: res(w, og.RenderSettings),
      fog: res(w, og.Fog),
      post: res(w, og.PostEffects),
      style: res(w, og.MaterialStyle),
      look: og.render.look(w) as unknown as Info | null,
    };
  });
  expect(atStart).toBeDefined();
  const s = atStart!;
  // The look's own fields.
  expect(s.stylise.pixelHeight).toBe(180);
  expect(s.stylise.posterize).toBe(6);
  expect(s.stylise.dither).toBe("bayer4");
  expect(s.stylise.palette).not.toBe(0);
  expect(s.style.toon).toBeTrue();
  expect(s.style.outlineWidth).toBe(2);
  expect(s.post.vignette).toBeTrue();
  expect(s.post.vignetteIntensity).toBeCloseTo(0.5, 6);
  // Its base's, with its own winning field by field.
  expect(s.settings.toneMapping).toBe("agx");
  expect(s.settings.exposure).toBeCloseTo(0.8, 6);
  expect(s.fog.enabled).toBeTrue();
  // "#808080" written linear by the cook.
  expect(s.fog.color.r).toBeCloseTo(0.2158605, 5);
  expect(s.fog.density).toBeCloseTo(0.02, 6);
  // Fields the look does not name, in a resource it does, are defaults.
  expect(s.stylise.ditherStrength).toBe(1);
  expect(s.post.outline).toBeFalse();
  const info = s.look!;
  expect(info.path).toBe("looks/dusk.ogt");
  expect(info.name).toBe("dusk");
  expect(info.extends).toEqual(["looks/base.ogt"]);
  expect(info.resources).toEqual(["RenderSettings", "PostEffects", "Stylise", "MaterialStyle", "Fog"]);
  expect(info.antialias).toBe("off");
  expect(info.skin).not.toBeNull();
  // The skin configures the UI from the next tick.
  expect(engine.advance(1).ok).toBeTrue();
  expect((og.ui.theme(engine.world) as unknown as { skin: string }).skin).toBe("tiny");
});

test("the look's antialias is the renderer's when GameDesc.render leaves it out", () => {
  const engine = start({ look: "looks/dusk.ogt" });
  const attached = JSON.parse(native().og_node_attach_offscreen(engine.id, 64, 64)) as { ok: boolean; reason?: string };
  if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  const caps = og.render.capabilities(engine.world) as unknown as { antialias: string; stylise: string[] };
  expect(caps.antialias).toBe("off");
  expect(Array.isArray(caps.stylise)).toBeTrue();
  const own = start({ look: "looks/dusk.ogt", render: { antialias: "msaa4" } });
  const again = JSON.parse(native().og_node_attach_offscreen(own.id, 64, 64)) as { ok: boolean };
  expect(again.ok).toBeTrue();
  expect((og.render.capabilities(own.world) as unknown as { antialias: string }).antialias).not.toBe("off");
});

test("applyLook replaces the resources it names whole and leaves the rest", () => {
  const engine = start({ look: "looks/dusk.ogt" });
  const w = engine.world;
  expect(engine.advance(1).ok).toBeTrue();
  // Game code animates the flash; a look keeps it.
  res(w, og.PostEffects).flash = 0.25;
  const bare = must(og.load.look(w, "looks/bare.ogt"));
  const info = must(og.load.lookInfo(w, bare)) as unknown as Info;
  expect(info.name).toBe("Bare");
  expect(info.resources).toEqual(["PostEffects", "Stylise"]);
  expect(info.skin).toBeNull();
  must(og.render.applyLook(w, bare));
  const stylise = res(w, og.Stylise);
  expect(stylise.posterize).toBe(0);
  expect(stylise.pixelHeight).toBe(0);
  expect(stylise.palette).toBe(0);
  const post = res(w, og.PostEffects);
  expect(post.vignette).toBeFalse();
  expect(post.flash).toBeCloseTo(0.25, 6);
  // Resources the bare look does not name keep the first look's values.
  expect(res(w, og.RenderSettings).toneMapping).toBe("agx");
  expect(res(w, og.MaterialStyle).toon).toBeTrue();
  expect((og.render.look(w) as unknown as Info).name).toBe("Bare");
  // Loading again answers the same handle; the first look applies again.
  const dusk = must(og.load.look(w, "looks/dusk.ogt"));
  expect(must(og.load.look(w, "assets/looks/dusk.ogt"))).toBe(dusk);
  must(og.render.applyLook(w, dusk));
  expect(res(w, og.Stylise).posterize).toBe(6);
});

test("palettes load once per path, and refusals name the fix", () => {
  const engine = start({});
  const w = engine.world;
  expect(og.render.look(w)).toBeNull();
  const p = must(og.load.palette(w, "palettes/dusk.hex"));
  expect(must(og.load.palette(w, "palettes/dusk.hex"))).toBe(p);
  res(w, og.Stylise).palette = p;
  expect(res(w, og.Stylise).palette).toBe(p);
  const missing = og.load.look(w, "looks/missing.ogt");
  expect(missing.ok).toBeFalse();
  if (!missing.ok) expect(missing.reason).toContain("og cook");
  const notLook = og.render.applyLook(w, 12345 as never);
  expect(notLook.ok).toBeFalse();
  const bad = start({ look: "looks/missing.ogt" });
  const log = bad.takeLog().map((l) => l.text);
  expect(log.some((t) => t.startsWith("GameDesc.look:") && t.includes("og cook"))).toBeTrue();
});
