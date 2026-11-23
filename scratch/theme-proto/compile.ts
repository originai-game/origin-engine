// Theme prototype compiler: a theme recipe (dimensions + `extends` + `with` mix-ins) → today's
// pack.json (seeds, theme, fonts, images, variants, sounds) + its art, under templates/styles/<out>/.
//
//   bun scratch/theme-proto/compile.ts <theme> [...]     (reads scratch/theme-proto/themes/<theme>.json)
//
// The dimension libraries below (MATERIALS, SHAPES, TYPE, MOTION, SOUND) stand in for the data
// files the design proposes (templates/themes/dimensions/*.json); every value a theme ends up with
// comes from a recipe or a dimension, never from this file's logic.
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { contrast, ensure, hexToOklch, Lch, toHex } from "./color.ts";

const here = import.meta.dir;
const repo = resolve(here, "../..");
type Recipe = Record<string, any>;

function deep(a: any, b: any): any {
  if (b === undefined) return a;
  if (a && b && typeof a === "object" && typeof b === "object" && !Array.isArray(a) && !Array.isArray(b)) {
    const o: any = { ...a };
    for (const k of Object.keys(b)) o[k] = deep(a[k], b[k]);
    return o;
  }
  return b;
}

/** extends first (recursively), then each `with` mix-in in order, then the recipe's own keys. */
export function resolveRecipe(name: string, seen: string[] = []): Recipe {
  if (seen.includes(name)) throw new Error(`extends cycle: ${[...seen, name].join(" → ")}`);
  const own: Recipe = JSON.parse(readFileSync(join(here, "themes", `${name}.json`), "utf8"));
  let base: Recipe = own.extends ? resolveRecipe(own.extends, [...seen, name]) : {};
  for (const m of own.with ?? []) base = deep(base, JSON.parse(readFileSync(join(here, "mixins", `${m}.json`), "utf8")));
  const { extends: _e, with: _w, ...rest } = own;
  return deep(base, rest);
}

// ---- dimension libraries -------------------------------------------------------------------
const L = (c: Lch, l: number, chroma?: number) => toHex({ ...c, l, c: chroma ?? c.c });

/** A material: how its faces are lit from the recipe's hues (lightness and chroma per surface). */
const MATERIALS: Record<string, (p: Lch, n: Lch, a: Lch) => Record<string, any>> = {
  glass: (p, n) => ({ button: [L(n, 0.3, 0.03), L(n, 0.2, 0.03)], panel: [L(n, 0.24, 0.03), L(n, 0.16, 0.03)], plate: [L(n, 0.24, 0.03), L(n, 0.15, 0.03)], chip: [L(n, 0.3, 0.04), L(n, 0.22, 0.04)], slot: [L(n, 0.14, 0.03), L(n, 0.2, 0.03)], field: [L(n, 0.13, 0.02), L(n, 0.18, 0.02)], outline: null, drop: 0.5, outline_w: 1 }),
  metal: (p, n) => ({ button: [L(n, 0.42, 0.025), L(n, 0.3, 0.025)], panel: [L(n, 0.3, 0.02), L(n, 0.22, 0.02)], plate: [L(n, 0.28, 0.02), L(n, 0.2, 0.02)], chip: [L(n, 0.36, 0.03), L(n, 0.28, 0.03)], slot: [L(n, 0.16, 0.02), L(n, 0.24, 0.02)], field: [L(n, 0.15, 0.015), L(n, 0.2, 0.015)], outline: L(n, 0.08, 0.01), rim_lo: L(n, 0.22, 0.02), rim_hi: L(n, 0.82, 0.02), drop: 0.5, outline_w: 1 }),
  paper: (p, n) => ({ button: [L(n, 0.95, 0.03), L(n, 0.89, 0.035)], panel: [L(n, 0.95, 0.03), L(n, 0.9, 0.035)], plate: [L(n, 0.93, 0.03), L(n, 0.88, 0.035)], chip: [L(n, 0.9, 0.04), L(n, 0.85, 0.045)], slot: [L(n, 0.84, 0.04), L(n, 0.9, 0.035)], field: [L(n, 0.97, 0.02), L(n, 0.94, 0.02)], outline: L(n, 0.3, 0.04), paper_shade: L(n, 0.55, 0.06), drop: 0.22, outline_w: 1.25 }),
  wood: (p, n) => ({ button: [L(n, 0.5, 0.08), L(n, 0.38, 0.08)], panel: [L(n, 0.4, 0.07), L(n, 0.3, 0.07)], plate: [L(n, 0.34, 0.06), L(n, 0.26, 0.06)], chip: [L(n, 0.46, 0.08), L(n, 0.36, 0.08)], slot: [L(n, 0.22, 0.05), L(n, 0.3, 0.05)], field: [L(n, 0.2, 0.04), L(n, 0.26, 0.04)], outline: L(n, 0.15, 0.04), grain: L(n, 0.18, 0.06), drop: 0.45, outline_w: 1.5 }),
  plastic: (p, n, a) => ({ button: [L(a, 0.84, 0.13), L(a, 0.7, 0.15)], panel: [L(n, 0.3, 0.12), L(n, 0.24, 0.12)], plate: [L(n, 0.32, 0.13), L(n, 0.26, 0.13)], chip: [L(n, 0.34, 0.14), L(n, 0.28, 0.14)], slot: [L(n, 0.28, 0.1), L(n, 0.36, 0.1)], field: [L(n, 0.25, 0.08), L(n, 0.31, 0.08)], outline: L(n, 0.18, 0.08), drop: 0.35, outline_w: 2.5, lip: 6 }),
  pixel: (p, n) => ({ button: [L(n, 0.5, 0.06), L(n, 0.4, 0.06)], panel: [L(n, 0.3, 0.06), L(n, 0.24, 0.06)], plate: [L(n, 0.26, 0.05), L(n, 0.2, 0.05)], chip: [L(n, 0.42, 0.06), L(n, 0.34, 0.06)], slot: [L(n, 0.18, 0.05), L(n, 0.24, 0.05)], field: [L(n, 0.16, 0.04), L(n, 0.2, 0.04)], outline: L(n, 0.1, 0.03), drop: 0, outline_w: 1 }),
};

const SHAPES: Record<string, { family: string; radius: number; cut?: number; theme: Record<string, number> }> = {
  sharp: { family: "rounded", radius: 2, theme: { control: 0, field: 0, block: 0, button: 0, card: 0, panel: 0, sheet: 0 } },
  soft: { family: "rounded", radius: 6, theme: { control: 4, field: 6, block: 6, button: 6, card: 8, panel: 10, sheet: 14 } },
  round: { family: "rounded", radius: 16, theme: { control: 10, field: 12, block: 14, button: 14, card: 18, panel: 22, sheet: 28 } },
  cut: { family: "cut", radius: 1, cut: 9, theme: { control: 0, field: 0, block: 0, button: 0, card: 0, panel: 0, sheet: 0 } },
  pixel: { family: "pixel", radius: 3, theme: { control: 0, field: 0, block: 0, button: 0, card: 0, panel: 0, sheet: 0 } },
};

const SH = "@styles/shared/", CJK = "@styles/cjk/";
const TYPE: Record<string, { fonts: string[]; copy?: string[]; family: Record<string, string> }> = {
  "storybook-serif": { fonts: [`${SH}Cinzel[wght].ttf`, `${SH}Newsreader[opsz,wght].ttf`, `${CJK}NotoSerifSC[wght].ttf`], family: { display: `"Cinzel", "Noto Serif SC"`, sans: `"Newsreader", "Noto Serif SC"`, mono: `"Newsreader", "Noto Serif SC"` } },
  "condensed-stencil": { fonts: [`${SH}BigShouldersStencilDisplay-SemiBold.ttf`, `${SH}BarlowSemiCondensed-SemiBold.ttf`, `${SH}Rajdhani-Bold.ttf`, `${CJK}NotoSansSC[wght].ttf`], family: { display: `"Big Shoulders Stencil Display", "Noto Sans SC"`, sans: `"Barlow Semi Condensed", "Noto Sans SC"`, mono: `"Rajdhani", "Noto Sans SC"` } },
  "rounded-display": { fonts: ["LilitaOne-Regular.ttf", "Fredoka[wdth,wght].ttf", `${CJK}ZCOOLKuaiLe-Regular.ttf`, `${CJK}NotoSansSC[wght].ttf`], copy: ["cartoon/LilitaOne-Regular.ttf", "cartoon/Fredoka[wdth,wght].ttf"], family: { display: `"Lilita One", "ZCOOL KuaiLe"`, sans: `"Fredoka", "Noto Sans SC"`, rounded: `"Fredoka", "Noto Sans SC"`, mono: `"Fredoka", "Noto Sans SC"` } },
  pixel: { fonts: [`${CJK}FusionPixel12-zhHans.ttf`], family: { display: `"Fusion Pixel 12px Prop zh-Hans"`, sans: `"Fusion Pixel 12px Prop zh-Hans"`, mono: `"Fusion Pixel 12px Prop zh-Hans"`, rounded: `"Fusion Pixel 12px Prop zh-Hans"` } },
};

/** A motion character: the motion slots' classes (Material's standard/expressive schemes, stepped for pixel). */
const MOTION: Record<string, Record<string, string>> = {
  calm: {
    "Press.feedback": "transition-transform ease-spring-[1600,80] active:scale-98 active:duration-120 active:ease-out",
    "Modal.frame": "transition ease-spring-[220,30] starting:opacity-0 starting:scale-98 exit:opacity-0 exit:duration-180 exit:ease-exit",
    "Screen.frame": "transition duration-400 ease-out starting:opacity-0 exit:opacity-0 exit:duration-180 exit:ease-exit",
    "List.cascade": "stagger-60/600",
  },
  snappy: {
    "Press.feedback": "transition-transform ease-spring-[3200,110] active:scale-96 active:duration-50 active:ease-out",
    "Modal.frame": "transition ease-spring-[520,46] starting:opacity-0 starting:scale-96 exit:opacity-0 exit:duration-120 exit:ease-exit",
    "List.cascade": "stagger-25/250",
  },
  bouncy: {
    "Press.feedback": "transition-transform ease-spring-[1800,22] active:scale-90 active:duration-50 active:ease-out",
    "Modal.frame": "transition ease-spring-[300,11] starting:opacity-0 starting:scale-75 exit:opacity-0 exit:scale-90 exit:duration-120 exit:ease-exit",
    "List.cascade": "stagger-50/500",
    "List.item": "animate-pop",
  },
  stepped: {
    "Press.feedback": "transition-transform duration-0 active:scale-96 active:duration-0",
    "Modal.frame": "transition duration-0 starting:opacity-0 exit:opacity-0 exit:duration-0",
    "Screen.frame": "transition duration-0 starting:opacity-0 exit:opacity-0 exit:duration-0",
    "List.cascade": "stagger-80/480",
  },
};

/** A sound character: one of the mastered clip sets (the design adds more; this proves the slot). */
const SOUND: Record<string, string | null> = { default: null, scifi: "scifi", fantasy: "fantasy", cartoon: "cartoon" };

const CLEAR = "bg-transparent hover:bg-transparent active:bg-transparent border-transparent outline-0 shadow-none";

// ---- compile -------------------------------------------------------------------------------
export function compile(name: string): { dir: string; recipe: Recipe } {
  const r = resolveRecipe(name);
  const out = join(repo, "templates", "styles", `proto-${name}`);
  rmSync(out, { recursive: true, force: true });
  mkdirSync(join(out, "art"), { recursive: true });
  const c = r.color;
  const p = hexToOklch(c.source), a = hexToOklch(c.accent ?? c.source), n = hexToOklch(c.neutral ?? c.source);
  const mat = MATERIALS[r.material]!(p, n, a);
  const shape = SHAPES[r.shape]!;
  // The primary and danger faces: the source's hue at the material's face lightness band.
  const band = r.material === "paper" ? [0.62, 0.5] : r.material === "plastic" ? [0.86, 0.74] : [0.66, 0.54];
  const face = (h: Lch) => [L(h, band[0]!, Math.max(h.c, 0.12)), L(h, band[1]!, Math.max(h.c, 0.12))];
  const danger = hexToOklch(c.danger ?? "#d8404c");
  const P = {
    ...mat,
    button: mat.button, primary: face(p), danger: face(danger),
    disabled: [L(n, 0.5, 0.01), L(n, 0.42, 0.01)],
    orn: toHex({ ...a, l: r.material === "paper" ? 0.45 : 0.78 }),
    header: L(p, 0.45, Math.max(p.c, 0.1)),
    shadow: "#000000",
  };
  const spec = {
    out: join(out, "art"), material: r.material, shape: { family: shape.family, radius: shape.radius, cut: shape.cut ?? 0 },
    ornament: { button: "none", panel: "none", plate: "none", slot: "none", ...r.ornament },
    palette: P, lip: mat.lip ?? 0, header: r.ornament?.header ?? false,
  };
  writeFileSync(join(out, "art.spec.json"), JSON.stringify(spec, null, 1));
  const run = Bun.spawnSync(["/tmp/tvenv/bin/python", join(here, "art.py"), join(out, "art.spec.json")]);
  if (run.exitCode !== 0) throw new Error(new TextDecoder().decode(run.stderr));
  const images = JSON.parse(new TextDecoder().decode(run.stdout));
  rmSync(join(out, "art.spec.json"));
  const ogt = readFileSync(join(repo, "templates/styles/shared/art/frames/button.png.ogt"), "utf8");
  for (const k of Object.keys(images)) writeFileSync(join(out, "art", `${k}.png.ogt`), ogt);

  // Inks: each art ground's ink, found by the contrast search (4.5:1 on every face of that art).
  const dark = (hex: string) => hexToOklch(hex).l < 0.6;
  const ink = (faces: string[], ratio = 4.5) => {
    const want = dark(faces[0]!) ? { ...n, l: 0.97, c: 0.015 } : { ...n, l: 0.22, c: 0.03 };
    return toHex(ensure(want, faces, ratio));
  };
  const artFg = ink([...P.plate, ...P.chip]);
  const buttonFg = ink(P.button);
  const primaryFg = ink(P.primary);
  const panelScheme = dark(P.panel[0]) ? "dark" : "light";
  const groundScheme = (f: string) => (dark(f) ? "dark" : "light");

  const motion = MOTION[r.motion] ?? {};
  const btn = (img: string, fg: string, s: string) =>
    `scheme-${s} nine-${img} hover:nine-${img}-hover active:nine-${img}-pressed disabled:nine-button-disabled ${CLEAR} text-${fg} data-pressed:text-${fg} data-pressed:hover:text-${fg} disabled:text-${fg}/55`;
  const variants: Record<string, string> = {
    "Button.plate": btn("button", "art-button-fg", groundScheme(P.button[0])),
    "Button.plate.primary": btn("button-primary", "art-primary-fg", groundScheme(P.primary[0])),
    "Panel.frame": `scheme-${panelScheme} nine-panel ${CLEAR} text-fg`,
    "Plate.frame": `scheme-${groundScheme(P.plate[0])} nine-plate ${CLEAR} text-art-fg`,
    "Chip.frame": `scheme-${groundScheme(P.chip[0])} nine-chip ${CLEAR} text-art-fg`,
    "Slot.frame": `nine-slot`,
    "Field.frame": `scheme-${groundScheme(P.field[0])} nine-field ${CLEAR} text-fg`,
    ...motion,
  };
  const type = TYPE[r.type]!;
  for (const f of type.copy ?? []) cpSync(join(repo, "templates/styles", f), join(out, f.split("/").pop()!));
  for (const f of type.copy ?? []) if (existsSync(join(repo, "templates/styles", f + ".ogt"))) cpSync(join(repo, "templates/styles", f + ".ogt"), join(out, f.split("/").pop()! + ".ogt"));
  // Ground registration: each art face a slot draws under text is a ground of the scheme the slot
  // names, and every ink role is solved (lightness only, hue and chroma kept) to hold its ratio on
  // all of that scheme's registered grounds as well as the design's own.
  const def = JSON.parse(readFileSync(join(repo, "crates/og-ui-next/default/pack.json"), "utf8")).theme.colors;
  const side = (v: any, s: string) => (typeof v === "string" ? v : v[s]);
  const grounds: Record<string, string[]> = { dark: [], light: [] };
  writeFileSync(join(out, "images.json"), JSON.stringify(images));
  const smp = Bun.spawnSync(["/tmp/tvenv/bin/python", "-I", join(here, "sample.py"), out, join(out, "images.json")]);
  rmSync(join(out, "images.json"));
  const faces: Record<string, string[]> = JSON.parse(new TextDecoder().decode(smp.stdout));
  for (const [img, s] of [["panel", panelScheme], ["plate", groundScheme(P.plate[0])], ["chip", groundScheme(P.chip[0])], ["field", groundScheme(P.field[0])], ["slot", groundScheme(P.slot[0])]] as const)
    grounds[s]!.push(...(faces[img] ?? []));
  for (const s of ["dark", "light"]) grounds[s]!.push(side(def.surface, s), side(def["surface-raised"], s), side(def.bg, s));
  const INKS: Array<[string, number]> = [["fg", 4.5], ["fg-muted", 4.5], ["accent-ink", 4.5], ["danger", 4.5], ["warn", 4.5], ["success", 4.5], ["info", 4.5], ["gold", 4.5], ["xp", 4.5], ["epic", 4.5], ["legendary", 4.5], ["mythic", 4.5], ["hud-accent-ink", 4.5], ["gold-hi", 3]];
  const solved: Record<string, any> = {};
  for (const [role, ratio] of INKS) {
    const pair: Record<string, string> = {};
    for (const s of ["dark", "light"]) pair[s] = toHex(ensure(hexToOklch(side(def[role], s)), grounds[s]!, ratio));
    solved[role] = pair;
  }
  const pack: Recipe = {
    about: `Compiled by scratch/theme-proto/compile.ts from the theme recipe \`${name}\`${r.extends ? "" : ""}.`,
    scheme: c.scheme ?? "dark",
    seeds: { primary: c.source, accent: c.accent ?? c.source, neutral: c.neutral ?? c.source, ...(c.danger ? { danger: c.danger } : {}) },
    theme: {
      colors: {
        ...solved,
        "art-fg": artFg, "art-button-fg": buttonFg, "art-primary": P.primary[0], "art-primary-fg": primaryFg,
        "icon-outline": mat.outline ?? L(n, 0.12, 0.02),
      },
      radius: shape.theme,
      fontFamily: type.family,
    },
    fonts: type.fonts,
    images,
    variants,
  };
  const snd = SOUND[r.sound ?? "default"];
  if (snd) {
    const src = JSON.parse(readFileSync(join(repo, "templates/styles", snd, "pack.json"), "utf8"));
    pack.sounds = src.sounds;
    cpSync(join(repo, "templates/styles", snd, "sfx"), join(out, "sfx"), { recursive: true });
  }
  writeFileSync(join(out, "pack.json"), JSON.stringify(pack, null, 2) + "\n");
  // The contrast facts the compile guarantees, for the report.
  const facts = {
    "art-button-fg on button": Math.min(...P.button.map((f: string) => contrast(buttonFg, f))).toFixed(2),
    "art-primary-fg on primary": Math.min(...P.primary.map((f: string) => contrast(primaryFg, f))).toFixed(2),
    "art-fg on plate/chip": Math.min(...[...P.plate, ...P.chip].map((f: string) => contrast(artFg, f))).toFixed(2),
  };
  writeFileSync(join(out, "README.md"), `# proto-${name}\n\nPrototype pack compiled from a theme recipe. Resolved recipe:\n\n\`\`\`json\n${JSON.stringify(r, null, 2)}\n\`\`\`\n\nContrast guaranteed at compile: ${JSON.stringify(facts)}\n`);
  return { dir: out, recipe: r };
}

if (import.meta.main) {
  for (const t of process.argv.slice(2)) {
    const { dir, recipe } = compile(t);
    console.log(dir, JSON.stringify(recipe));
  }
}
