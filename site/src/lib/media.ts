import index from "../generated/media.json";
import { media as slots, type SlotName } from "../media";
import type { Lang } from "../i18n";

export interface Resolved {
  kind: "image" | "video" | "pending";
  status: "final" | "interim" | "pending";
  width?: number;
  height?: number;
  duration?: number;
  avif?: [string, number][];
  webp?: [string, number][];
  jpg?: string;
  mp4?: string;
  poster?: string;
  from: { id?: string; path?: string; commit?: string; date?: string; machine?: string; pending?: string };
}

/** Where public/m/ is served from: the site itself in preview, R2 on Cloudflare. */
const BASE = (import.meta.env.PUBLIC_MEDIA_BASE as string | undefined) ?? "/m/";
export const url = (file: string) => `${BASE}${file}`;

export function get(name: SlotName): Resolved {
  const r = (index as Record<string, Resolved>)[name];
  if (!r) throw new Error(`media: slot ${name} is not in src/generated/media.json; run bun scripts/media.ts`);
  return r;
}

export const alt = (name: SlotName, lang: Lang) => slots[name].alt[lang];

export function srcset(list: [string, number][] | undefined) {
  return (list ?? []).map(([f, w]) => `${url(f)} ${w}w`).join(", ");
}

/** The provenance line: archive id or path, commit, date, machine. */
export function provenance(r: Resolved): string {
  const f = r.from;
  if (f.pending) return f.pending;
  return [f.id ?? f.path, f.commit, f.date, f.machine].filter(Boolean).join(" · ");
}
