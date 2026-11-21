import { CHOICE_LABELS, localized, LOOK_ZH } from "./choice-labels.ts";

/** Creation vocabulary, defaults and inference rules. */
export const CREATION_CHOICES = {
  form: ["2d", "2.5d", "3d", "film"],
  genre: ["interactive-film", "action-platformer", "shooter", "rpg", "puzzle-match", "strategy-tower-defense", "racing", "runner", "roguelike-card", "idle", "rhythm", "arena", "open-world"],
  view: ["side-2d", "top-down-2d", "isometric-2.5d", "third-person-3d", "first-person-3d"],
  ui: ["default", "cartoon", "fantasy", "scifi"],
  platform: ["phone-portrait", "phone-landscape", "desktop", "multi"],
  input: ["touch", "keyboard-mouse", "gamepad"],
  players: ["single", "local-multiplayer", "online-lockstep"],
  music: ["chiptune", "orchestral", "electronic", "ambient", "folk", "rock"],
} as const;

type ChoiceKey = keyof typeof CREATION_CHOICES;
type Choice<K extends ChoiceKey> = (typeof CREATION_CHOICES)[K][number];
export type Form = Choice<"form">;
export type View = Choice<"view">;
export type Genre = Choice<"genre">;

/** The fields a film leaves out: a film is footage paced by a Yarn story, not a game with a camera. */
export const GAME_ONLY = ["view", "players"] as const;
/** A film's genre, and only a film's: `--genre interactive-film` makes the form film. */
export const FILM_GENRE = "interactive-film";
type GameOnly = (typeof GAME_ONLY)[number];

/** Each form's views; a game's view is one of its form's, and a film has none. */
export const FORM_VIEWS: { readonly [F in Form]: readonly View[] } = {
  "2d": ["side-2d", "top-down-2d"],
  "2.5d": ["isometric-2.5d"],
  "3d": ["third-person-3d", "first-person-3d"],
  film: [],
};

/** Each genre's views, preferred first: a game's default view is the first of its form's. */
export const GENRE_VIEWS: { readonly [G in Genre]: readonly View[] } = {
  "interactive-film": [],
  "action-platformer": ["side-2d", "third-person-3d", "isometric-2.5d", "top-down-2d", "first-person-3d"],
  shooter: ["third-person-3d", "first-person-3d", "top-down-2d", "side-2d", "isometric-2.5d"],
  rpg: ["isometric-2.5d", "top-down-2d", "third-person-3d", "side-2d", "first-person-3d"],
  "puzzle-match": ["top-down-2d", "isometric-2.5d", "side-2d", "third-person-3d", "first-person-3d"],
  "strategy-tower-defense": ["isometric-2.5d", "top-down-2d", "third-person-3d", "side-2d", "first-person-3d"],
  racing: ["third-person-3d", "top-down-2d", "isometric-2.5d", "first-person-3d", "side-2d"],
  runner: ["side-2d", "third-person-3d", "top-down-2d", "isometric-2.5d", "first-person-3d"],
  "roguelike-card": ["top-down-2d", "side-2d", "isometric-2.5d", "third-person-3d", "first-person-3d"],
  idle: ["top-down-2d", "side-2d", "isometric-2.5d", "third-person-3d", "first-person-3d"],
  rhythm: ["side-2d", "top-down-2d", "third-person-3d", "isometric-2.5d", "first-person-3d"],
  arena: ["top-down-2d", "isometric-2.5d", "third-person-3d", "side-2d", "first-person-3d"],
  "open-world": ["third-person-3d", "isometric-2.5d", "top-down-2d", "first-person-3d", "side-2d"],
};

/** The form a view belongs to. */
export function formOf(view: View): Exclude<Form, "film"> {
  return view.endsWith("-2d") ? "2d" : view.endsWith("-2.5d") ? "2.5d" : "3d";
}

/** The fields that apply to a form, in the create block's order. */
export function formFields(form: Form): string[] {
  return ["form", "genre", "view", "look", "variant", "ui", "platform", "input", "players", "lang", "music"]
    .filter(k => form !== "film" || !(GAME_ONLY as readonly string[]).includes(k));
}

/** A game's default view: its genre's first view of its form. */
export function defaultView(genre: Genre, form: Exclude<Form, "film">, views: readonly string[] = FORM_VIEWS[form]): View | undefined {
  return GENRE_VIEWS[genre].find(v => formOf(v) === form && views.includes(v));
}

export type CreationChoices = { readonly [K in Exclude<ChoiceKey, GameOnly>]: Choice<K> } & { readonly [K in GameOnly]?: Choice<K> } & {
  readonly look: string;
  /** One of the look's variants (docs/art.md §2.11); absent: the look as it is. */
  readonly variant?: string;
  readonly lang: readonly string[];
};
export type CreationInput = { -readonly [K in keyof CreationChoices]?: CreationChoices[K] };

const RULES: Partial<Record<ChoiceKey, readonly (readonly [string, RegExp])[]>> = {
  form: [["film", /interactive.(film|movie)|\bfmv\b|互动影游|互动电影|影游/i]],
  genre: [["interactive-film", /interactive.(film|movie)|\bfmv\b|互动影游|互动电影|影游/i], ["arena", /arena|竞技|对战/i], ["shooter", /shoot|\bfps\b|射击/i], ["rpg", /\brpg\b|role.play|角色扮演/i], ["puzzle-match", /puzzle|match.?3|解谜|消除/i], ["strategy-tower-defense", /strategy|tower.defen|策略|塔防/i], ["racing", /rac(e|ing)|赛车/i], ["runner", /runner|跑酷/i], ["roguelike-card", /rogue|deck|card|卡牌/i], ["idle", /idle|incremental|放置/i], ["rhythm", /rhythm|节奏/i], ["open-world", /open.world|探索/i], ["action-platformer", /platform|action|动作|跳跃/i]],
  view: [["isometric-2.5d", /isometric|2\.5d|等距/i], ["first-person-3d", /first.person|\bfps\b|第一人称/i], ["third-person-3d", /third.person|\b3d\b|第三人称/i], ["top-down-2d", /top.down|俯视/i], ["side-2d", /side.scroll|platform|\b2d\b|横版|跳跃/i]],
  platform: [["phone-landscape", /phone.landscape|手机横屏/i], ["phone-portrait", /portrait|mobile|phone|手机|竖屏/i], ["multi", /multi.platform|cross.platform|多端/i]],
  input: [["gamepad", /gamepad|controller|手柄/i], ["touch", /touch|触屏/i]],
  players: [["online-lockstep", /online|lockstep|联网/i], ["local-multiplayer", /local.multiplayer|couch|同屏/i]],
  music: [["chiptune", /chiptune|芯片/i], ["orchestral", /orchestra|管弦/i], ["electronic", /electronic|电子/i], ["folk", /folk|民族/i], ["rock", /\brock\b|摇滚/i]],
};

const SLUG = "^[a-z0-9]+(-[a-z0-9]+)*$";

/** JSON Schema for the platform's creation form. Look slugs come from the existing catalogue. */
export function creationSchema(looks: readonly string[]) {
  return {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    type: "object",
    additionalProperties: false,
    required: formFields("film").filter(k => k !== "variant"),
    properties: {
      ...Object.fromEntries(Object.entries(CHOICE_LABELS).map(([key, labels]) => [key, {
        type: "string", enum: Object.keys(labels),
        "x-labels": Object.fromEntries(Object.entries(labels).map(([value, copy]) => [value, localized(copy)])),
      }])),
      look: { type: "string", enum: looks, "x-labels": Object.fromEntries(looks.map(slug => {
        const copy = LOOK_ZH[slug];
        return [slug, copy ? { label: { "zh-CN": copy[0] }, description: { "zh-CN": copy[1] } } : {}];
      })) },
      variant: { type: "string", pattern: SLUG, description: "One of the look's variants (the document's compat.looks[look].variants)." },
      lang: { type: "array", minItems: 1, uniqueItems: true, items: { type: "string", pattern: "^[A-Za-z]{2,3}(-[A-Za-z0-9]{2,8})*$" } },
    },
    // A film's genre is interactive-film and it has no view or players; a game has all three, its genre another, its view one of its form's.
    allOf: [
      { if: { properties: { form: { const: "film" } } }, then: { properties: { genre: { const: FILM_GENRE } }, not: { anyOf: GAME_ONLY.map(k => ({ required: [k] })) } }, else: { required: [...GAME_ONLY], properties: { genre: { not: { const: FILM_GENRE } } } } },
      ...(["2d", "2.5d", "3d"] as const).map(form => ({ if: { properties: { form: { const: form } } }, then: { properties: { view: { enum: [...FORM_VIEWS[form]] } } } })),
    ],
  };
}

/** Validate a partial selection at the API/CLI boundary; explicit choices always win inference. */
export function parseCreationInput(value: unknown): CreationInput {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("creation choices must be an object");
  const result: Record<string, unknown> = {};
  for (const [key, item] of Object.entries(value)) {
    if (key === "mood") {
      // Removed from the creation choices (the owner, 2026-11-20): a look draws in its own declared mood.
      throw new Error('mood is no longer a creation choice: a look draws in its own mood (its preset\'s or variant\'s); to change it, write mood "<word>" in the game\'s look file');
    } else if (key === "look" || key === "variant") {
      if (typeof item !== "string" || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(item)) throw new Error(`${key} must be a catalogue slug`);
    } else if (key === "lang") {
      if (!Array.isArray(item) || !item.length || item.some(v => typeof v !== "string" || !/^[A-Za-z]{2,3}(?:-[A-Za-z0-9]{2,8})*$/.test(v)) || new Set(item).size !== item.length) throw new Error("lang must be a nonempty list of distinct language tags");
    } else {
      const entry = Object.entries(CREATION_CHOICES).find(([name]) => name === key);
      if (!entry || typeof item !== "string" || !entry[1].some(v => v === item)) throw new Error(`${key}: choose ${entry?.[1].join(", ") ?? "a known creation field"}`);
    }
    result[key] = item;
  }
  return result as CreationInput;
}

/**
 * Deterministic suggestions, not an LLM: the caller confirms them and supplies the chosen look's UI.
 * The form comes from an explicit form or view, then the premise, then `defaults`; a game's view is
 * one of its form's (its genre's first when nothing names one). A film's genre is interactive-film; it has no view or players.
 */
export function inferCreationChoices(premise: string, input: CreationInput = {}, defaults: CreationInput = {}): CreationChoices {
  const inferred: Record<string, string> = {};
  for (const [key, rules] of Object.entries(RULES)) {
    const match = rules.find(([, pattern]) => pattern.test(premise));
    if (match) inferred[key] = match[0];
  }
  const validated = parseCreationInput(input), fallback = parseCreationInput(defaults), suggestions = parseCreationInput(inferred);
  if (validated.form && validated.view && formOf(validated.view) !== validated.form) throw new Error(`view ${validated.view} is ${formOf(validated.view)}, not form ${validated.form}; a ${validated.form === "film" ? "film has no view" : `${validated.form} view is one of ${FORM_VIEWS[validated.form].join(", ")}`}`);
  const platform = validated.platform ?? suggestions.platform ?? fallback.platform ?? "desktop";
  const players = validated.players ?? suggestions.players ?? fallback.players ?? "single";
  const viewForm = (v?: View) => v && formOf(v);
  if (validated.genre === FILM_GENRE && validated.form !== undefined && validated.form !== "film") throw new Error(`genre ${FILM_GENRE} is form film, not ${validated.form}`);
  const form: Form = validated.form ?? (validated.genre === FILM_GENRE ? "film" : undefined) ?? viewForm(validated.view) ?? suggestions.form ?? viewForm(suggestions.view) ?? fallback.form ?? viewForm(fallback.view) ?? (players === "single" ? "3d" : "2d");
  if (form === "film") for (const k of GAME_ONLY) if (validated[k] !== undefined) throw new Error(`${k} does not apply to a film (form film): leave it out`);
  if (form === "film" && validated.genre !== undefined && validated.genre !== FILM_GENRE) throw new Error(`a film's genre is ${FILM_GENRE}, not ${validated.genre}`);
  if (form !== "film" && validated.genre === FILM_GENRE) throw new Error(`genre ${FILM_GENRE} is form film, not ${form}`);
  const lang = validated.lang ?? suggestions.lang ?? fallback.lang ?? (/[\u3040-\u30ff]/u.test(premise) ? ["ja"] : /[\u4e00-\u9fff]/u.test(premise) ? ["zh"] : ["en"]);
  const genre = [validated.genre, suggestions.genre, fallback.genre].find(g => g !== undefined && g !== FILM_GENRE) ?? "action-platformer";
  const game = form === "film" ? { genre: FILM_GENRE } : {
    genre,
    view: validated.view ?? [suggestions.view, fallback.view].find(v => v && formOf(v) === form) ?? defaultView(genre, form)!,
  };
  const pick = <K extends "look" | "ui" | "music" | "input">(k: K, base: CreationChoices[K]) => validated[k] ?? suggestions[k] ?? fallback[k] ?? base;
  const variant = validated.variant ?? fallback.variant;
  return {
    form, ...game, look: pick("look", form === "film" ? "cinematic-realism" : "faceted-low-poly"), ...(variant === undefined ? {} : { variant }),
    ui: pick("ui", "default"), platform,
    input: pick("input", platform.startsWith("phone-") ? "touch" : "keyboard-mouse"),
    ...(form === "film" ? {} : { players }), lang, music: pick("music", "ambient"),
  };
}

/** The canonical project.ogt block: constraints and hints, not runtime settings. */
export function creationBlock(choices: CreationChoices): string {
  const checked = inferCreationChoices("", choices);
  return `create {\n${Object.entries(checked).map(([key, value]) => `  ${key} ${Array.isArray(value) ? value.map(v => JSON.stringify(v)).join(" ") : JSON.stringify(value)}`).join("\n")}\n}\n`;
}
