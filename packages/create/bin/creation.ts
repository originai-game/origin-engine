import { CHOICE_LABELS, localized, LOOK_ZH, VARIANT_ZH } from "./choice-labels.ts";
import { CREATION_CHOICES, creationSchema, defaultView, FORM_VIEWS, CREATION_FIELDS, formOf, GENRE_VIEWS } from "./choices.ts";
import type { Form, View } from "./choices.ts";
import { catalogueFile, creationLooks, readCatalogue, resolveCreationChoices } from "./create.ts";
import type { Style } from "./create.ts";

/**
 * The creation document (packages/create/README.md §Creation document): every
 * option a home page offers, with its labels, preview slot, compat rules and
 * defaults. A release publishes it as `<version>/creation.json` with every
 * preview slot filled; `og create --creation` prints it.
 */

/** A picture an option shows, by its path under the version root. */
export interface PreviewSlot {
  readonly path: string;
  readonly kind: "look" | "variant" | "ui";
  /** The look it is drawn in: the look or variant's own. */
  readonly look?: string;
  readonly variant?: string;
  readonly pack?: string;
}

/** What a release fills a slot with. */
export interface PreviewFile {
  readonly url: string;
  readonly sha256: string;
  readonly size: number;
  /** The engine version that rendered it. */
  readonly engine: string;
}

function styles(catalogue: string): Style[] {
  const all = readCatalogue(catalogue), ready = new Set(creationLooks(all));
  return all.filter(s => ready.has(s.slug));
}

/** Every preview the document references, in document order. */
export function previewSlots(catalogue: string = catalogueFile()): PreviewSlot[] {
  return [
    ...styles(catalogue).flatMap(s => [
      { path: `previews/${s.slug}.webp`, kind: "look" as const, look: s.slug },
      ...(s.variants ?? []).map(v => ({ path: `previews/${s.slug}.${v.name}.webp`, kind: "variant" as const, look: s.slug, variant: v.name })),
    ]),
    ...CREATION_CHOICES.ui.map(pack => ({ path: `previews/ui/${pack}.webp`, kind: "ui" as const, pack })),
  ];
}

/** The forms a look draws: those of its views. */
export function lookForms(style: Style): Form[] {
  return CREATION_CHOICES.form.filter(f => style.look.views.some(v => formOf(v as View) === f));
}

export interface CreationCompat {
  /** The views of each form. */
  readonly formViews: typeof FORM_VIEWS;
  /** Each genre's views, preferred first: its default view in a form is the first of that form's. */
  readonly genreViews: typeof GENRE_VIEWS;
  readonly genreDefaultView: Record<string, Partial<Record<Form, View>>>;
  /** The create block's fields, in its order; `view` may be left out by an interactive film. */
  readonly fields: readonly string[];
  readonly looks: Record<string, { forms: Form[]; views: readonly string[]; ui: string; variants: string[] }>;
}

/** The rules between options, as data. */
export function creationCompat(catalogue: string = catalogueFile()): CreationCompat {
  return {
    formViews: FORM_VIEWS,
    genreViews: GENRE_VIEWS,
    genreDefaultView: Object.fromEntries(CREATION_CHOICES.genre.map(g => [g, Object.fromEntries(CREATION_CHOICES.form.map(f => [f, defaultView(g, f)]))])),
    fields: CREATION_FIELDS,
    looks: Object.fromEntries(styles(catalogue).map(s => [s.slug, { forms: lookForms(s), views: s.look.views, ui: s.ui, variants: (s.variants ?? []).map(v => v.name) }])),
  };
}

export type CreationDocument = ReturnType<typeof creationDocument>;

/**
 * The whole document. Pure and deterministic: the same catalogue and
 * previews give the same bytes. With `previews`, each slot's path must be
 * filled and no other path given, or it throws; without, a slot is `{ path }`.
 */
export function creationDocument(previews?: Readonly<Record<string, PreviewFile>>, catalogue: string = catalogueFile()) {
  const slots = previewSlots(catalogue);
  if (previews) {
    const known = new Set(slots.map(s => s.path));
    const missing = slots.filter(s => !previews[s.path]).map(s => s.path);
    if (missing.length) throw new Error(`creation document: ${missing.length} preview slot(s) unfilled: ${missing.join(", ")}`);
    const extra = Object.keys(previews).filter(p => !known.has(p));
    if (extra.length) throw new Error(`creation document: preview(s) for no slot: ${extra.join(", ")}`);
  }
  const preview = (path: string) => {
    const f = previews?.[path];
    return f ? { path, url: f.url, sha256: f.sha256, size: f.size, engine: f.engine } : { path };
  };
  const all = styles(catalogue);
  const enumOptions = (key: keyof typeof CHOICE_LABELS, slot?: (id: string) => string) =>
    Object.entries(CHOICE_LABELS[key]).map(([id, copy]) => ({ id, ...localized(copy), ...(slot ? { preview: preview(slot(id)) } : {}) }));
  const text = (en: string, zh: string | undefined) => (zh === undefined ? { en } : { en, "zh-CN": zh });
  return {
    schemaVersion: 1,
    /** The home page's questions, in order; `more` holds the rest behind "more settings". */
    order: ["form", "genre", "look", "variant", "ui"],
    more: ["view", "platform", "input", "players", "lang", "music"],
    schema: creationSchema(all.map(s => s.slug)),
    options: {
      form: enumOptions("form"),
      genre: enumOptions("genre"),
      view: enumOptions("view").map(o => ({ ...o, form: formOf(o.id as View) })),
      look: all.map(s => ({
        id: s.slug,
        label: text(s.name, LOOK_ZH[s.slug]?.[0]),
        description: text(s.descriptor, LOOK_ZH[s.slug]?.[1]),
        preview: preview(`previews/${s.slug}.webp`),
        variants: (s.variants ?? []).map(v => ({
          id: v.name,
          label: text(v.title, VARIANT_ZH[`${s.slug}.${v.name}`]?.[0]),
          description: text(v.descriptor, VARIANT_ZH[`${s.slug}.${v.name}`]?.[1]),
          preview: preview(`previews/${s.slug}.${v.name}.webp`),
        })),
      })),
      ui: enumOptions("ui", id => `previews/ui/${id}.webp`),
      platform: enumOptions("platform"),
      input: enumOptions("input"),
      players: enumOptions("players"),
      music: enumOptions("music"),
    },
    compat: creationCompat(catalogue),
    /**
     * What `og create` picks with no premise, per form. With a premise it
     * infers from English and Chinese keywords; explicit choices win, the
     * look's UI pack is the UI default, and languages
     * follow the premise's script (kana: ja, Han: zh, else en).
     */
    defaults: {
      form: resolveCreationChoices("", {}, catalogue).form,
      byForm: Object.fromEntries(CREATION_CHOICES.form.map(f => [f, resolveCreationChoices("", { form: f }, undefined, catalogue)])),
    },
  };
}
