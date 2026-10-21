// Fetches the Japanese font's source, Noto Sans JP Regular (SIL OFL 1.1,
// notofonts/noto-cjk), into tools/fonts/ (not committed: 4.5 MB), checking
// its SHA-256. tools/subset-font.py then cuts the game's subset from it
// (assets/fonts/NotoSansJP-kestrel.otf, committed), as og-i18n §3 says:
//
//   bun tools/fetch-font.ts
//   python3 tools/subset-font.py tools/fonts/NotoSansJP-Regular.otf assets/fonts/NotoSansJP-kestrel.otf --locale ja --cjk-only

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const URL_JP = "https://github.com/notofonts/noto-cjk/raw/main/Sans/SubsetOTF/JP/NotoSansJP-Regular.otf";
const SHA256 = "dff723ba59d57d136764a04b9b2d03205544f7cd785a711442d6d2d085ac5073";
const out = join(resolve(import.meta.dir ?? ".", "fonts"), "NotoSansJP-Regular.otf");

const sum = (b: Uint8Array) => createHash("sha256").update(b).digest("hex");
if (existsSync(out) && sum(readFileSync(out)) === SHA256) {
  console.log(`${out} is there`);
} else {
  const res = await fetch(URL_JP);
  if (!res.ok) throw new Error(`${URL_JP}: ${res.status}`);
  const bytes = new Uint8Array(await res.arrayBuffer());
  if (sum(bytes) !== SHA256) throw new Error(`${URL_JP} changed: sha256 ${sum(bytes)}, expected ${SHA256}`);
  mkdirSync(join(out, ".."), { recursive: true });
  writeFileSync(out, bytes);
  console.log(`fetched ${out}`);
}
