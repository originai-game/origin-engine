// Chrome's reference text sheets: the layout of sheet/src/main.tsx in each pack's
// `sans` family list (its pack.json theme), with the default pack's body face and
// the Chinese face behind it as og's fallback chain has them, and the body text's line
// height the UI root passes down.
// usage: bun chrome-sheets.ts <chrome> <out-dir> [packs, default default,cartoon,fantasy,scifi]
// Writes <out-dir>/chrome-<pack>.html and chrome-<pack>.png (1200x620, DPR 1).
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [chrome, outArg, packArg] = process.argv.slice(2);
if (!chrome || !outArg) {
  console.error("usage: bun chrome-sheets.ts <chrome> <out-dir> [packs]");
  process.exit(2);
}
const out = resolve(outArg);
mkdirSync(out, { recursive: true });
const styles = resolve(import.meta.dir, "../../templates/styles");
const packs = (packArg ?? "default,cartoon,fantasy,scifi").split(",");
const SIZES = [12, 14, 16, 20, 28, 48];
const LINES = ["Settings · Quit to menu · Hamburgefonstiv", "开始游戏 设置 第 3 关 · Level 3", "0123456789 +1,240 XP 06:54 ×320"];

/** Each family the packs' sans lists name, with its files and their weight ranges. */
const FACES: Record<string, Array<[string, string]>> = {
  "Instrument Sans": [["shared/InstrumentSans[wdth,wght].ttf", "400 700"]],
  Fredoka: [["cartoon/Fredoka[wdth,wght].ttf", "300 700"]],
  Newsreader: [["shared/Newsreader[opsz,wght].ttf", "200 800"]],
  Spectral: [["shared/Spectral-SemiBold.ttf", "600"]],
  "Barlow Semi Condensed": [["shared/BarlowSemiCondensed-SemiBold.ttf", "600"]],
  "Chakra Petch": [["shared/ChakraPetch-SemiBold.ttf", "600"], ["shared/ChakraPetch-Bold.ttf", "700"]],
  "Noto Sans SC": [["cjk/NotoSansSC[wght].ttf", "100 900"]],
  "Noto Serif SC": [["cjk/NotoSerifSC[wght].ttf", "200 900"]],
};

/** The line height the UI root passes down with the theme's body text (`text-body`): a
 * unitless ratio, which every text inherits as a multiple of its own size, as from a
 * page's body. The pack's `fontSize.body` over the default pack's. */
function bodyLine(pack: string): number {
  const of = (file: string) => JSON.parse(readFileSync(file, "utf8")).theme?.fontSize?.body as [number, number] | undefined;
  const own = pack === "default" ? undefined : of(join(styles, pack, "pack.json"));
  return (own ?? of(resolve(import.meta.dir, "../../crates/og-ui-next/default/pack.json"))!)[1];
}

function sans(pack: string): string {
  if (pack === "default") return `"Instrument Sans"`;
  const theme = JSON.parse(readFileSync(join(styles, pack, "pack.json"), "utf8")).theme;
  return theme.fontFamily.sans as string;
}

let failed = 0;
for (const pack of packs) {
  // og's chain after the pack's own families: the default body face, then Chinese.
  const list = `${sans(pack)}, "Instrument Sans", "Noto Sans SC"`;
  // A pack.json whose `fonts` declare families gives its @font-face rules itself;
  // one that lists files names its families by the files' own names (FACES).
  const declared = pack === "default" ? [] : ((JSON.parse(readFileSync(join(styles, pack, "pack.json"), "utf8")).fonts ?? []) as Array<string | { family: string; src: string; weight?: number | [number, number]; unicodeRange?: string }>).filter((f) => typeof f === "object") as Array<{ family: string; src: string; weight?: number | [number, number]; unicodeRange?: string }>;
  // A file the checkout lacks (templates not fetched: bun scripts/assets.ts ensure templates)
  // would leave Chrome drawing a system font, measured as og's error.
  const need = (path: string) => {
    if (!existsSync(path)) throw new Error(`${pack}: ${path} is missing (bun scripts/assets.ts ensure templates)`);
    return pathToFileURL(path).href;
  };
  const src = (f: string) => need(f.startsWith("@styles/") ? join(styles, f.slice(8)) : join(styles, pack, f));
  const own = new Set(declared.map((f) => f.family));
  const names = [...list.matchAll(/"([^"]+)"/g)].map((m) => m[1]!).filter((n) => !own.has(n));
  const faces = [
    ...declared.map((f) => `@font-face{font-family:"${f.family}";src:url("${src(f.src)}")${f.weight === undefined ? "" : `;font-weight:${[f.weight].flat().join(" ")}`}${f.unicodeRange ? `;unicode-range:${f.unicodeRange}` : ""}}`),
    ...names.flatMap((n) => {
      if (!FACES[n]) throw new Error(`${pack}: no file for family "${n}" in chrome-sheets.ts FACES`);
      return FACES[n]!.map(([file, w]) => `@font-face{font-family:"${n}";src:url("${need(join(styles, file))}");font-weight:${w}}`);
    }),
  ]
    .join("\n");
  const rows = LINES.flatMap((l) => SIZES.map((s) => `<div class="r" style="height:${Math.round(s * 1.3)}px"><span class="l">${s}</span><span class="t" style="font-size:${s}px">${l}</span></div>`)).join("\n");
  const html = `<!doctype html><meta charset="utf-8"><style>
${faces}
html,body{margin:0;background:#141a22;width:1200px;height:620px;overflow:hidden}
body{padding:8px 12px;box-sizing:border-box;font-family:${list};font-weight:400;line-height:${bodyLine(pack)};color:#e8ecf2}
.r{display:flex;align-items:center}.l{font-size:10px;color:rgba(255,255,255,.5);width:28px;flex:none}.t{white-space:nowrap}
</style>${rows}`;
  const page = join(out, `chrome-${pack}.html`);
  const png = join(out, `chrome-${pack}.png`);
  writeFileSync(page, html);
  const p = Bun.spawnSync([chrome, "--headless=new", "--no-sandbox", "--hide-scrollbars", "--allow-file-access-from-files", "--force-device-scale-factor=1", "--window-size=1200,620", "--virtual-time-budget=3000", `--screenshot=${png}`, pathToFileURL(page).href], { stdout: "ignore", stderr: "pipe" });
  if (p.exitCode !== 0 || !existsSync(png)) {
    console.error(`chrome-sheets: ${pack}: Chrome wrote no screenshot: ${p.stderr.toString().slice(-400)}`);
    failed++;
  } else console.log(`chrome-sheets: ${png}`);
}
process.exit(failed ? 1 : 0);
