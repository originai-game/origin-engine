import { CHOICE_LABELS, localized, LOOK_ZH } from "./choice-labels.ts";

/** Creation vocabulary, defaults and inference rules. */
export const CREATION_CHOICES = {
  genre: ["action-platformer", "shooter", "rpg", "puzzle-match", "strategy-tower-defense", "racing", "runner", "roguelike-card", "idle", "rhythm", "arena", "open-world"],
  view: ["side-2d", "top-down-2d", "isometric-2.5d", "third-person-3d", "first-person-3d"],
  mood: ["bright", "dark", "warm", "cold", "soft", "high-contrast"],
  ui: ["default"],
  platform: ["phone-portrait", "phone-landscape", "desktop", "multi"],
  input: ["touch", "keyboard-mouse", "gamepad"],
  players: ["single", "local-multiplayer", "online-lockstep"],
  music: ["chiptune", "orchestral", "electronic", "ambient", "folk", "rock"],
} as const;

type ChoiceKey = keyof typeof CREATION_CHOICES;
export type CreationChoices = { readonly [K in ChoiceKey]: (typeof CREATION_CHOICES)[K][number] } & {
  readonly look: string;
  readonly lang: readonly string[];
};
export type CreationInput = Partial<CreationChoices>;

const RULES: Partial<Record<ChoiceKey, readonly (readonly [string, RegExp])[]>> = {
  genre: [["arena", /arena|竞技|对战/i], ["shooter", /shoot|\bfps\b|射击/i], ["rpg", /\brpg\b|role.play|角色扮演/i], ["puzzle-match", /puzzle|match.?3|解谜|消除/i], ["strategy-tower-defense", /strategy|tower.defen|策略|塔防/i], ["racing", /rac(e|ing)|赛车/i], ["runner", /runner|跑酷/i], ["roguelike-card", /rogue|deck|card|卡牌/i], ["idle", /idle|incremental|放置/i], ["rhythm", /rhythm|节奏/i], ["open-world", /open.world|探索/i], ["action-platformer", /platform|action|动作|跳跃/i]],
  view: [["isometric-2.5d", /isometric|2\.5d|等距/i], ["first-person-3d", /first.person|\bfps\b|第一人称/i], ["third-person-3d", /third.person|\b3d\b|第三人称/i], ["top-down-2d", /top.down|俯视/i], ["side-2d", /side.scroll|platform|\b2d\b|横版|跳跃/i]],
  mood: [["high-contrast", /high.contrast|高对比/i], ["dark", /dark|horror|阴暗|恐怖/i], ["cold", /cold|icy|冷峻|冰雪/i], ["warm", /warm|温暖/i], ["soft", /soft|cosy|cozy|柔和/i], ["bright", /bright|明亮/i]],
  platform: [["phone-landscape", /phone.landscape|手机横屏/i], ["phone-portrait", /portrait|mobile|phone|手机|竖屏/i], ["multi", /multi.platform|cross.platform|多端/i]],
  input: [["gamepad", /gamepad|controller|手柄/i], ["touch", /touch|触屏/i]],
  players: [["online-lockstep", /online|lockstep|联网/i], ["local-multiplayer", /local.multiplayer|couch|同屏/i]],
  music: [["chiptune", /chiptune|芯片/i], ["orchestral", /orchestra|管弦/i], ["electronic", /electronic|电子/i], ["folk", /folk|民族/i], ["rock", /\brock\b|摇滚/i]],
};

/** JSON Schema for the platform's creation form. Look slugs come from the existing catalogue. */
export function creationSchema(looks: readonly string[]) {
  return {
    $schema: "https://json-schema.org/draft/2020-12/schema",
    type: "object",
    additionalProperties: false,
    required: [...Object.keys(CREATION_CHOICES), "look", "lang"],
    properties: {
      ...Object.fromEntries(Object.entries(CHOICE_LABELS).map(([key, labels]) => [key, {
        type: "string", enum: Object.keys(labels),
        "x-labels": Object.fromEntries(Object.entries(labels).map(([value, copy]) => [value, localized(copy)])),
      }])),
      look: { type: "string", enum: looks, "x-labels": Object.fromEntries(looks.map(slug => {
        const copy = LOOK_ZH[slug];
        return [slug, copy ? { label: { "zh-CN": copy[0] }, description: { "zh-CN": copy[1] } } : {}];
      })) },
      lang: { type: "array", minItems: 1, uniqueItems: true, items: { type: "string", pattern: "^[A-Za-z]{2,3}(-[A-Za-z0-9]{2,8})*$" } },
    },
  };
}

/** Validate a partial selection at the API/CLI boundary; explicit choices always win inference. */
export function parseCreationInput(value: unknown): CreationInput {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("creation choices must be an object");
  const result: Record<string, unknown> = {};
  for (const [key, item] of Object.entries(value)) {
    if (key === "look") {
      if (typeof item !== "string" || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(item)) throw new Error("look must be a catalogue slug");
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

/** Deterministic suggestions, not an LLM: the caller confirms them and supplies the chosen look's UI. */
export function inferCreationChoices(premise: string, input: CreationInput = {}, defaults: CreationInput = {}): CreationChoices {
  const inferred: Record<string, string> = {};
  for (const [key, rules] of Object.entries(RULES)) {
    const match = rules.find(([, pattern]) => pattern.test(premise));
    if (match) inferred[key] = match[0];
  }
  const validated = parseCreationInput(input);
  const platform = validated.platform ?? inferred.platform ?? defaults.platform ?? "desktop";
  const players = validated.players ?? inferred.players ?? defaults.players ?? "single";
  const base: CreationChoices = {
    genre: "action-platformer", view: players === "single" ? "third-person-3d" : "top-down-2d", look: "faceted-low-poly",
    mood: "bright", ui: "default", platform: "desktop", input: "keyboard-mouse",
    players: "single", lang: /[\u3040-\u30ff]/u.test(premise) ? ["ja"] : /[\u4e00-\u9fff]/u.test(premise) ? ["zh"] : ["en"], music: "ambient",
  };
  const suggestions = parseCreationInput(inferred);
  return { ...base, ...parseCreationInput(defaults), ...suggestions, input: suggestions.input ?? defaults.input ?? (platform.startsWith("phone-") ? "touch" : "keyboard-mouse"), ...validated };
}

/** The canonical project.ogt block: constraints and hints, not runtime settings. */
export function creationBlock(choices: CreationChoices): string {
  const checked = inferCreationChoices("", choices);
  return `create {\n${Object.entries(checked).map(([key, value]) => `  ${key} ${Array.isArray(value) ? value.map(v => JSON.stringify(v)).join(" ") : JSON.stringify(value)}`).join("\n")}\n}\n`;
}
