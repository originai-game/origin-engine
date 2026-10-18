// Build-time brand for the player shell: the marks and the motion rules
// `og build` inlines into index.html, derived from the vendored brand files
// (shell/brand/, copied from the platform's packages/brand; README there).
// Nothing here is drawn: every path comes from origin-mark.svg.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const BRAND_DIR = join(dirname(fileURLToPath(import.meta.url)), "brand");

/** The motions the shell uses: boot, real progress, error. */
export const SHELL_MOTIONS = ["genesis", "progress", "drift"] as const;

function brandFile(name: string): string {
  return readFileSync(join(BRAND_DIR, name), "utf8");
}

interface Parts {
  viewBox: string;
  outer: { transform: string; paths: string };
  inner: { transform: string; paths: string };
  hub: string;
}

function parts(svg: string): Parts {
  const arm = (name: string) => {
    const m = new RegExp(`<g data-part="${name}"><g transform="([^"]+)">([\\s\\S]*?)</g></g>`).exec(svg);
    if (!m) throw new Error(`origin-mark.svg lacks the ${name} arms`);
    return { transform: m[1]!, paths: m[2]! };
  };
  const hub = /<g data-part="hub">([\s\S]*?)<\/g>/.exec(svg)?.[1];
  const viewBox = /viewBox="([^"]+)"/.exec(svg)?.[1];
  if (!hub || !viewBox) throw new Error("origin-mark.svg lacks the hub or the viewBox");
  return { viewBox, outer: arm("outer"), inner: arm("inner"), hub };
}

/**
 * The boot mark: one symbol that shows `genesis` (the layered arms and the
 * hub), `progress` (a faint ghost and a wedge-masked copy whose wipe the
 * shell moves to 100 − percent) and `drift`, by switching `data-motion`.
 */
export function bootMark(svg = brandFile("origin-mark.svg")): string {
  const p = parts(svg);
  const both = '<use href="#og-mo"/><use href="#og-mi"/>';
  return `<svg class="og-mark og-boot-mark" data-motion="genesis" viewBox="${p.viewBox}" aria-hidden="true" focusable="false">`
    + `<defs><g id="og-mo" transform="${p.outer.transform}">${p.outer.paths}</g><g id="og-mi" transform="${p.inner.transform}">${p.inner.paths}</g>`
    + '<mask id="og-wipe" maskUnits="userSpaceOnUse" x="-270" y="-270" width="540" height="540">'
    + '<circle data-part="wipe" r="100" fill="none" stroke="#fff" stroke-width="204" pathLength="100" stroke-dasharray="100 100" stroke-dashoffset="100" transform="rotate(-100)"/></mask></defs>'
    + '<g data-part="arms"><g data-part="outer"><use href="#og-mo"/></g><g data-part="inner"><use href="#og-mi"/></g></g>'
    + `<g data-part="ghost">${both}</g><g data-part="fill" mask="url(#og-wipe)">${both}</g>`
    + `<g data-part="hub">${p.hub}</g></svg>`;
}

/** The `engine` product mark (the symbol's inner arms and core ring), still, for "Made with Origin Engine". */
export function engineMark(svg = brandFile("origin-mark.svg")): string {
  const p = parts(svg);
  return `<svg class="og-engine-mark" viewBox="${p.viewBox}" fill="currentColor" aria-hidden="true" focusable="false">`
    + `<g transform="${p.inner.transform}">${p.inner.paths}</g>${p.hub}</svg>`;
}

function splitTopLevel(text: string): string[] {
  const out: string[] = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === "(" || c === "[") depth++;
    else if (c === ")" || c === "]") depth--;
    else if (c === "," && depth === 0) {
      out.push(text.slice(start, i));
      start = i + 1;
    }
  }
  out.push(text.slice(start));
  return out.map((s) => s.trim()).filter(Boolean);
}

function blocks(css: string): Array<{ prelude: string; body: string }> {
  const out: Array<{ prelude: string; body: string }> = [];
  let depth = 0;
  let start = 0;
  let open = -1;
  for (let i = 0; i < css.length; i++) {
    if (css[i] === "{") {
      if (depth === 0) open = i;
      depth++;
    } else if (css[i] === "}") {
      depth--;
      if (depth === 0) {
        out.push({ prelude: css.slice(start, open).trim(), body: css.slice(open + 1, i) });
        start = i + 1;
      }
    }
  }
  return out;
}

/**
 * The rules of origin-motion.css the given motions need: the base
 * `.og-mark` rules, every rule naming only those motions, their keyframes
 * and the reduced-motion block for the same selectors.
 */
export function motionSubset(motions: readonly string[], source = brandFile("origin-motion.css")): string {
  const css = source.replace(/\/\*[\s\S]*?\*\//g, "");
  const keep = (selector: string) => {
    if (!selector.includes(".og-mark")) return false;
    const named = [...selector.matchAll(/data-motion="([a-z]+)"/g)].map((m) => m[1]!);
    return named.length ? named.every((n) => motions.includes(n)) : !/data-part="ring"/.test(selector);
  };
  const rules = (text: string): string[] =>
    blocks(text).flatMap(({ prelude, body }) => {
      if (prelude.startsWith("@keyframes")) return [];
      if (prelude.startsWith("@media")) {
        const inner = rules(body);
        return inner.length ? [`${prelude}{${inner.join("")}}`] : [];
      }
      const selectors = splitTopLevel(prelude).filter(keep);
      return selectors.length ? [`${selectors.join(",")}{${body.trim().replace(/\s+/g, " ")}}`] : [];
    });
  const kept = rules(css);
  const used = new Set(kept.join("").match(/og-[a-z-]+(?=[\s;}])/g) ?? []);
  const frames = blocks(css)
    .filter(({ prelude }) => prelude.startsWith("@keyframes") && used.has(prelude.split(/\s+/)[1]!))
    .map(({ prelude, body }) => `${prelude}{${body.trim().replace(/\s+/g, " ")}}`);
  return [...kept, ...frames].join("\n");
}

/** The shell's token values (brand/tokens.css). */
export function shellTokens(): string {
  return brandFile("tokens.css").replace(/\/\*[\s\S]*?\*\//g, "").trim();
}
