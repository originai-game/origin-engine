// The brand marks, from the engine's vendored copy of the platform's brand
// (packages/engine/shell/brand/), built by the same functions `og build`
// uses (shell/brand.ts). The files are read here and passed in, because the
// bundled page cannot find them relative to brand.ts. Several marks share
// the page, so each gets its own ids.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { bootMark, engineMark, motionSubset } from "../../../packages/engine/shell/brand.ts";

const dir = join(process.cwd(), "../packages/engine/shell/brand");
const svg = readFileSync(join(dir, "origin-mark.svg"), "utf8");
const css = readFileSync(join(dir, "origin-motion.css"), "utf8");

export function mark(motion: "genesis" | "progress" | "answer" | "none", id: string, extra = ""): string {
  return bootMark(svg)
    .replaceAll("og-mo", `og-mo-${id}`)
    .replaceAll("og-mi", `og-mi-${id}`)
    .replaceAll("og-wipe", `og-wipe-${id}`)
    .replace('data-motion="genesis"', `data-motion="${motion}" ${extra}`);
}

export const engine = () => engineMark(svg);
export const motionCss = () => motionSubset(["genesis", "progress", "answer"], css);
