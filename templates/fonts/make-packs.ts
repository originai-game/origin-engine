// Builds Noto script packs, declarations, licences and hyphenation dictionaries.
//   bun templates/fonts/make-packs.ts [--cache DIR] [--pack NAME ...] [--out DIR]
// Requires HarfBuzz's hb-subset (libharfbuzz-bin). Unlike fontTools it is
// native; binary packing differs, while axes, characters and layout are kept.
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";

type Pack = {
  description: string;
  scripts: string[];
  languages: string[];
  faces: { file: string; source: string; wanted: Set<number> | null }[];
};
const range = (a: number, b: number) => Array.from({ length: b - a + 1 }, (_, i) => a + i);
function codec(name: string, lead: number[], trail: number[]): number[] {
  // iconv's strict national encodings, not WHATWG's Windows-compatible aliases.
  // Newlines isolate invalid pairs; discard their possible ASCII trailing byte.
  const bytes = lead.flatMap((hi) => trail.flatMap((lo) => [hi, lo, 10]));
  const r = Bun.spawnSync(["iconv", "-c", "-f", name, "-t", "UTF-8"], { stdin: Uint8Array.from(bytes) });
  if (r.exitCode) throw new Error(r.stderr.toString());
  const out = new Set([...r.stdout.toString()].map((c) => c.codePointAt(0)!).filter((c) => c > 127));
  if (name === "BIG5") {
    // GNU iconv calls its CP950-derived table BIG5: its C6A1..C6FE are
    // private-use instead of Big5's kana, and several symbols are CP950's.
    // Explicitly select the strict Big5 repertoire used to build the pins.
    for (const c of [
      ...range(0xf6b1, 0xf70e),
      0xaf,
      0x2027,
      0x20ac,
      0x2215,
      0x2295,
      0x2299,
      0xfe51,
      0xfe68,
      0xff5e,
      0xffe0,
      0xffe1,
      0xffe5,
    ])
      out.delete(c);
    for (const c of [
      ...range(0x3041, 0x3093),
      ...range(0x30a1, 0x30a7),
      0xa2,
      0xa3,
      0xa5,
      0x2022,
      0x203e,
      0x223c,
      0x2609,
      0x2641,
      0x3005,
      0x309d,
      0x309e,
      0x30fe,
      0xff64,
    ])
      out.add(c);
  } else if (name === "EUC-KR") {
    // A4D4 starts a multi-byte Hangul composition, not a standalone scalar;
    // A2E8 is outside KS X 1001 (iconv accepts its later postal-symbol addendum).
    out.delete(0x3164);
    out.delete(0x327e);
  }
  return [...out];
}
const ASCII = range(0x20, 0x7e),
  CJK = [
    ...range(0x3000, 0x303f),
    ...range(0xff01, 0xff60),
    ...range(0xffe0, 0xffe6),
    0x2018,
    0x2019,
    0x201c,
    0x201d,
    0x2026,
    0x2025,
    0x30fb,
  ];
const subset = (...ranges: number[][]) => new Set(ranges.flat());
const national = (encoding: string, leads: number[], extra: number[] = [], trails = range(0xa1, 0xfe)) =>
  subset(codec(encoding, leads, trails), CJK, extra);
const LATIN = subset(
  ASCII,
  range(0xa0, 0x24f),
  range(0x300, 0x36f),
  range(0x1e00, 0x1eff),
  range(0x2000, 0x206f),
  range(0x20a0, 0x20cf),
  [0x2122, 0x2190, 0x2192],
);
const CYRILLIC_GREEK = subset(range(0x370, 0x3ff), range(0x400, 0x4ff), range(0x500, 0x52f), range(0x1f00, 0x1fff));
const JIS = national("EUC-JP", [...range(0xa1, 0xa8), ...range(0xb0, 0xcf)]);
const KSX = national("EUC-KR", [...range(0xa1, 0xac), ...range(0xb0, 0xc8)], range(0x3131, 0x318e));
const BIG5 = national("BIG5", range(0xa1, 0xc6), range(0x3100, 0x312f), [...range(0x40, 0x7e), ...range(0xa1, 0xfe)]);
const GB = national("GB2312", [...range(0xa1, 0xa9), ...range(0xb0, 0xf7)]);
const PACKS: Record<string, Pack> = Object.fromEntries(
  (
    [
      [
        "latin",
        "Noto Sans for the Latin a style's face lacks: Latin-1, Latin Extended A and B, Vietnamese, combining marks",
        "Latn",
        "en es fr de it pt nl sv da no nb nn fi is pl cs sk sl hr bs ro hu tr az vi id ms tl fil sw et lv lt sq mt ga cy eu ca gl af ha yo ig zu xh uz tk la eo",
        "NotoSans-Latin.ttf",
        "notosans/NotoSans[wdth,wght].ttf",
        LATIN,
      ],
      [
        "cyrillic-greek",
        "Noto Sans Cyrillic, Cyrillic Supplement, Greek and Extended Greek",
        "Cyrl Grek",
        "ru uk be bg sr mk kk ky mn tg el",
        "NotoSans-CyrillicGreek.ttf",
        "notosans/NotoSans[wdth,wght].ttf",
        CYRILLIC_GREEK,
      ],
      [
        "arabic",
        "Noto Sans Arabic: Arabic with its joining forms and digits, Persian, Urdu",
        "Arab",
        "ar fa ur ps sd ug ckb",
        "NotoSansArabic-Regular.ttf",
        "notosansarabic/NotoSansArabic[wdth,wght].ttf",
        null,
      ],
      [
        "hebrew",
        "Noto Sans Hebrew with its points",
        "Hebr",
        "he iw yi",
        "NotoSansHebrew-Regular.ttf",
        "notosanshebrew/NotoSansHebrew[wdth,wght].ttf",
        null,
      ],
      [
        "devanagari",
        "Noto Sans Devanagari with its conjuncts: Hindi, Marathi, Nepali, Sanskrit",
        "Deva",
        "hi mr ne sa",
        "NotoSansDevanagari-Regular.ttf",
        "notosansdevanagari/NotoSansDevanagari[wdth,wght].ttf",
        null,
      ],
      ["thai", "Noto Sans Thai with its marks", "Thai", "th", "NotoSansThai-Regular.ttf", "notosansthai/NotoSansThai[wdth,wght].ttf", null],
      [
        "japanese",
        "Noto Sans JP: JIS X 0208 rows 1-8 and level 1 kanji (2,965), kana, CJK and fullwidth punctuation",
        "Jpan",
        "ja",
        "NotoSansJP-Regular.ttf",
        "notosansjp/NotoSansJP[wght].ttf",
        JIS,
      ],
      [
        "korean",
        "Noto Sans KR: KS X 1001 rows 1-12 and the 2,350 hangul syllables, compatibility jamo",
        "Kore",
        "ko",
        "NotoSansKR-Regular.ttf",
        "notosanskr/NotoSansKR[wght].ttf",
        KSX,
      ],
      [
        "chinese-traditional",
        "Noto Sans TC: Big5 symbols and level 1 hanzi (5,401), bopomofo",
        "Hant",
        "zh-Hant zh-TW zh-HK zh-MO",
        "NotoSansTC-Regular.ttf",
        "notosanstc/NotoSansTC[wght].ttf",
        BIG5,
      ],
      [
        "chinese-simplified",
        "Noto Sans SC: GB 2312 whole, its symbols and both levels of hanzi (6,763)",
        "Hans",
        "zh zh-Hans zh-CN zh-SG",
        "NotoSansSC-Regular.ttf",
        "notosanssc/NotoSansSC[wght].ttf",
        GB,
      ],
      [
        "emoji",
        "Noto Color Emoji (COLRv1): every emoji with its sequences (skin tones, ZWJ families, flags), painted in colour when first shown",
        "Zsye",
        "",
        "Noto-COLRv1.ttf",
        "noto-emoji:2D/fonts/Noto-COLRv1.ttf",
        null,
      ],
    ] satisfies [string, string, string, string, string, string, Set<number> | null][]
  ).map(([name, description, scripts, languages, file, source, wanted]) => [
    name,
    { description, scripts: scripts.split(" "), languages: languages ? languages.split(" ") : [], faces: [{ file, source, wanted }] },
  ]),
);

const args = process.argv.slice(2),
  selected: string[] = [];
let cache = ".amp/in/fonts-src",
  HERE = import.meta.dir;
while (args.length) {
  const flag = args.shift();
  if (flag === "--cache") cache = args.shift()!;
  else if (flag === "--out") HERE = args.shift()!;
  else if (flag === "--pack") selected.push(args.shift()!);
  else throw new Error(`unknown option ${flag}`);
}
const RAW = "https://github.com/google/fonts/raw/main/ofl/",
  EMOJI = "https://raw.githubusercontent.com/googlefonts/noto-emoji/main/";
const url = (source: string) =>
  source.startsWith("noto-emoji:") ? EMOJI + source.split(":")[1] : RAW + source.split("/").map(encodeURIComponent).join("/");
const family = (source: string) => (source.startsWith("noto-emoji:") ? "notoemoji" : source.split("/")[0]!);
async function fetchFile(path: string, url: string): Promise<void> {
  if (existsSync(path)) return;
  mkdirSync(dirname(path), { recursive: true });
  console.log(`fetch ${url}`);
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${url}: HTTP ${r.status}`);
  writeFileSync(path, new Uint8Array(await r.arrayBuffer()));
}
// fontTools' default bidi closure keeps the mirrored counterpart of every
// selected character. HarfBuzz keeps glyph closure but not those cmap entries.
// Use the same Unicode 16 mapping (Unicode data licence in the source header).
const bidiFile = join(cache, "BidiMirroring-16.0.0.txt");
await fetchFile(bidiFile, "https://www.unicode.org/Public/16.0.0/ucd/BidiMirroring.txt");
const mirrors = new Map(
  [...readFileSync(bidiFile, "utf8").matchAll(/^([0-9A-F]+);\s*([0-9A-F]+)/gm)].map(([, a, b]) => [parseInt(a!, 16), parseInt(b!, 16)]),
);
function table(font: Buffer, tag: string): Buffer | undefined {
  for (let i = 0; i < font.readUInt16BE(4); i++) {
    const at = 12 + i * 16;
    if (font.toString("ascii", at, at + 4) === tag)
      return font.subarray(font.readUInt32BE(at + 8), font.readUInt32BE(at + 8) + font.readUInt32BE(at + 12));
  }
}
function characters(font: Buffer): Set<number> {
  const cmap = table(font, "cmap")!,
    maps: { offset: number; priority: number }[] = [];
  // fontTools getBestCmap: Windows full repertoire, Unicode full repertoire,
  // Windows BMP, Unicode BMP, ISO Unicode, then the older Unicode encodings.
  const preference = [
    [3, 10],
    [0, 6],
    [0, 4],
    [3, 1],
    [0, 3],
    [0, 2],
    [0, 1],
    [0, 0],
  ];
  for (let i = 0; i < cmap.readUInt16BE(2); i++) {
    const at = 4 + i * 8,
      platform = cmap.readUInt16BE(at),
      encoding = cmap.readUInt16BE(at + 2);
    const priority = preference.findIndex(([p, e]) => platform === p && encoding === e);
    if (priority >= 0) maps.push({ offset: cmap.readUInt32BE(at + 4), priority });
  }
  const offset = maps.sort((a, b) => a.priority - b.priority)[0]!.offset,
    data = cmap.subarray(offset),
    fmt = data.readUInt16BE(),
    out = new Set<number>();
  if (fmt === 12 || fmt === 13) {
    for (let i = 0; i < data.readUInt32BE(12); i++) {
      const at = 16 + i * 12,
        start = data.readUInt32BE(at),
        end = data.readUInt32BE(at + 4),
        gid = data.readUInt32BE(at + 8);
      for (let c = start; c <= end; c++) if (gid + (fmt === 12 ? c - start : 0)) out.add(c);
    }
  } else if (fmt === 4) {
    const n = data.readUInt16BE(6) / 2;
    for (let i = 0; i < n; i++) {
      const end = data.readUInt16BE(14 + i * 2),
        start = data.readUInt16BE(16 + n * 2 + i * 2),
        delta = data.readInt16BE(16 + n * 4 + i * 2),
        at = 16 + n * 6 + i * 2,
        ro = data.readUInt16BE(at);
      for (let c = start; c <= end && c < 65535; c++) {
        const gid = ro ? data.readUInt16BE(at + ro + (c - start) * 2) : (c + delta) & 65535;
        if (gid) out.add(c);
      }
    }
  } else throw new Error(`unsupported cmap format ${fmt}`);
  return out;
}
function axes(font: Buffer): string[] {
  const fvar = table(font, "fvar");
  if (!fvar) return [];
  const start = fvar.readUInt16BE(4),
    count = fvar.readUInt16BE(8),
    size = fvar.readUInt16BE(10);
  return Array.from({ length: count }, (_, i) => {
    const at = start + i * size,
      tag = fvar.toString("ascii", at, at + 4),
      value = tag === "wght" ? 400 : tag === "wdth" ? 100 : fvar.readInt32BE(at + 8) / 65536;
    return `${tag}=${value}`;
  });
}
function tar(path: string, entry?: string): string | Buffer {
  const r = Bun.spawnSync(["tar", ...(entry ? ["-xOf", path, entry] : ["-tf", path])], { maxBuffer: 16 * 1024 * 1024 });
  if (r.exitCode) throw new Error(r.stderr.toString());
  return entry ? r.stdout : r.stdout.toString();
}
async function dictionaries(out: string, languages: string[]): Promise<string[]> {
  const crate = join(cache, "hypher-0.1.8.crate");
  await fetchFile(crate, "https://static.crates.io/crates/hypher/hypher-0.1.8.crate");
  const entries = (tar(crate) as string).split("\n").filter((p) => p.startsWith("hypher-0.1.8/tries/"));
  const tries = new Map(entries.map((p) => [basename(p).replace(/\.[^.]+$/, ""), p])),
    files: string[] = [];
  const invisible = new Set("as bn gu hi kn ml mr or pa sa ta te".split(" "));
  for (const language of languages) {
    const code = ["nb", "nn"].includes(language) ? "no" : language,
      file = `${code}.hyph`;
    if (tries.has(code) && !invisible.has(code) && !files.includes(file)) {
      writeFileSync(join(out, file), tar(crate, tries.get(code)) as Buffer);
      files.push(file);
    }
  }
  if (files.length) writeFileSync(join(out, "../../..", "LICENSE-hypher.txt"), tar(crate, "hypher-0.1.8/LICENSE-MIT") as Buffer);
  return files;
}
mkdirSync(HERE, { recursive: true });
const rows: string[] = [],
  kept = new Map<string, string[]>();
if (selected.length && existsSync(join(HERE, "LICENSES.md")))
  for (const line of readFileSync(join(HERE, "LICENSES.md"), "utf8").split("\n"))
    if (line.startsWith("| `")) {
      const key = line.slice(3).split("/")[0]!,
        r = kept.get(key) ?? [];
      r.push(line);
      kept.set(key, r);
    }
for (const [name, { description, scripts, languages, faces }] of Object.entries(PACKS)) {
  if (selected.length && !selected.includes(name)) {
    rows.push(...(kept.get(name) ?? []));
    continue;
  }
  const out = join(HERE, name),
    assets = join(out, "assets/fonts", name),
    content = join(out, "content/fonts", name);
  // Only replace the generator-owned output for this selected pack; never
  // remove the asset manifest, ignore rules or unrelated author files.
  mkdirSync(assets, { recursive: true });
  mkdirSync(content, { recursive: true });
  for (const dir of [assets, content]) for (const file of readdirSync(dir)) rmSync(join(dir, file), { recursive: true });
  const files: string[] = [];
  for (const { file, source, wanted } of faces) {
    const input = join(cache, basename(source));
    await fetchFile(input, url(source));
    const font = readFileSync(input),
      cmap = characters(font),
      requested = wanted ? [...wanted].filter((c) => cmap.has(c)) : [...cmap];
    const points = [
      ...new Set([
        ...requested,
        ...requested.flatMap((c) => {
          const mirror = mirrors.get(c);
          return mirror !== undefined && cmap.has(mirror) ? [mirror] : [];
        }),
      ]),
    ];
    const location = axes(font),
      output = join(assets, file),
      unicodeFile = join(cache, `${name}.unicodes`);
    writeFileSync(
      unicodeFile,
      points
        .sort((a, b) => a - b)
        .map((c) => c.toString(16))
        .join(","),
    );
    try {
      const r = Bun.spawnSync([
        "hb-subset",
        input,
        `--output-file=${output}`,
        `--unicodes-file=${unicodeFile}`,
        "--layout-features=*",
        "--layout-scripts=*",
        "--name-IDs=*",
        "--name-languages=*",
        "--name-legacy",
        "--notdef-outline",
        "--no-hinting",
        ...(location.length ? [`--instance=${location.join(" ")}`] : []),
      ]);
      if (r.exitCode) throw new Error(r.stderr.toString());
    } finally {
      rmSync(unicodeFile, { force: true });
    }
    const declaration = { kind: "font", source: `assets/fonts/${name}/${file}` };
    writeFileSync(
      join(content, file.replace(/\.[^.]+$/, ".ts")),
      "import {file} from '@origingame/engine/author';\n\n" + `export default file.import(${JSON.stringify(declaration, null, 2)});\n`,
    );
    const ofl = `OFL-${family(source)}.txt`,
      licence = join(cache, ofl);
    await fetchFile(
      licence,
      source.startsWith("noto-emoji:") ? EMOJI + dirname(source.split(":")[1]!) + "/LICENSE" : RAW + family(source) + "/OFL.txt",
    );
    writeFileSync(join(out, ofl), readFileSync(licence));
    const result = readFileSync(output),
      count = characters(result).size,
      size = result.length;
    const origin = source.startsWith("noto-emoji:")
      ? `googlefonts/noto-emoji \`${source.split(":")[1]}\``
      : `google/fonts \`ofl/${source}\``;
    rows.push(
      `| \`${name}/${file}\` | ${languages.join(", ") || "any (emoji)"} | ${count.toLocaleString("en-US")} characters, ${size.toLocaleString("en-US")} bytes | SIL OFL 1.1 (\`${ofl}\`) | ${origin} |`,
    );
    files.push(file);
    console.log(`${name}/${file}: ${size} bytes, ${count} characters`);
  }
  const hyphenation = await dictionaries(assets, languages);
  const declaration = {
    kind: "font",
    name,
    languages,
    scripts,
    faces: [],
    hyphenation: hyphenation.map((f) => `assets/fonts/${name}/${f}`),
  };
  const refs = files.map((f) => `ref(${JSON.stringify(`base/fonts/${name}/${f.replace(/\.[^.]+$/, "")}`)})`).join(", ");
  const text = JSON.stringify(declaration, null, 2).replace('"faces": []', `"faces": [${refs}]`);
  writeFileSync(
    join(content, "index.ts"),
    "import {file, ref} from '@origingame/engine/author';\n\n" + `export default file['font-extension'](${text});\n`,
  );
  writeFileSync(
    join(out, "extension.json"),
    JSON.stringify({ format: 1, kind: "font", name, version: "1.0.0", description, scripts, languages }, null, 2) + "\n",
  );
}
writeFileSync(
  join(HERE, "LICENSES.md"),
  `# Font packs: licences

Made by \`make-packs.ts\` from google/fonts: each a Noto face at Regular (wght
400, wdth 100), every layout feature kept, no hinting. Copyright The Noto
Project Authors; the CJK faces Copyright 2014-2021 Adobe, with Reserved Font
Name 'Source'. The emoji face is Noto Color Emoji's COLRv1 build from
googlefonts/noto-emoji, Copyright 2013 Google LLC. Each pack carries its licence file, which \`og font add\` and
\`og create --lang\` install with it.

A pack's \`<language>.hyph\` files are hyphenation dictionaries for CSS
\`hyphens: auto\`: hyph-utf8's TeX patterns as compiled by hypher 0.1.8
(crates.io \`hypher\`, MIT or Apache-2.0, \`LICENSE-hypher.txt\`; the patterns
are under their own permissive licences, LPPL, MPL, MIT or BSD-3, listed in
hypher's \`patterns/\`). The cook ships one only for a locale the game has
and only when its documents declare \`hyphens: auto\`.

| File | Languages | Holds | Licence | Source |
|---|---|---|---|---|
${rows.join("\n")}
`,
);
