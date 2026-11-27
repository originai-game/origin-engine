// Single source of truth for game creation choices, bilingual labels,
// compatibility rules, inference, and JSON Schema.
//
// `packages/knowledge/scripts/validate.ts` holds `CREATION_CHOICES.ui`,
// `LOOK_ZH`, and `VARIANT_ZH` equal to `looks/catalogue.json` and
// `templates/themes/`, and `@origingame/create-game` re-exports everything
// from here (`packages/create/bin/knowledge.ts`) without copying it.

import { MUSIC_STYLES } from "./brief.ts";

/** Creation vocabulary, defaults and inference rules. */
export const CREATION_CHOICES = {
  form: ["2d", "2.5d", "3d"],
  genre: ["interactive-film", "action-platformer", "shooter", "rpg", "puzzle-match", "strategy-tower-defense", "racing", "runner", "roguelike-card", "idle", "rhythm", "arena", "open-world"],
  view: ["side-2d", "top-down-2d", "isometric-2.5d", "third-person-3d", "first-person-3d"],
  ui: ["default", "anime", "candy", "cartoon", "fantasy", "ink", "minimal", "pixel", "scifi", "storybook", "tactical", "tactical-arctic"],
  platform: ["phone-portrait", "phone-landscape", "desktop", "multi"],
  input: ["touch", "keyboard-mouse", "gamepad"],
  players: ["single", "local-multiplayer", "online-lockstep"],
  music: MUSIC_STYLES,
} as const;

type ChoiceKey = keyof typeof CREATION_CHOICES;
type Choice<K extends ChoiceKey> = (typeof CREATION_CHOICES)[K][number];
export type Form = Choice<"form">;
export type View = Choice<"view">;
export type Genre = Choice<"genre">;

type Copy = readonly [enLabel: string, zhLabel: string, enDescription: string, zhDescription: string];

/** Form copy keyed by stable IDs; adding a choice requires both translations. */
export const CHOICE_LABELS = {
  form: {
    "2d": ["2D", "2D", "Flat sprites and tiles, seen from the side or from above.", "平面的精灵与瓦片，从侧面或上方观察。"],
    "2.5d": ["2.5D", "2.5D", "A 3D world seen through a fixed angled camera.", "通过固定斜角镜头观察的三维世界。"],
    "3d": ["3D", "3D", "A 3D world the camera moves through.", "镜头可在其中自由移动的三维世界。"],
  },
  genre: {
    "interactive-film": ["Interactive film", "互动影游", "A story told in segments that branch on the player's choices: footage, live scenes and playable levels.", "随玩家选择分支的影像剧情（形式为互动影游）。"],
    "action-platformer": ["Action platformer", "动作平台", "Run, jump and fight through obstacles.", "奔跑、跳跃并战胜沿途的障碍。"],
    shooter: ["Shooter", "射击", "Aim and shoot while managing movement and cover.", "瞄准射击，配合移动与掩护。"],
    rpg: ["Role-playing", "角色扮演", "Develop characters through quests and combat.", "通过任务与战斗培养角色。"],
    "puzzle-match": ["Puzzle and matching", "解谜消除", "Solve spatial puzzles or match pieces.", "解决空间谜题或匹配消除方块。"],
    "strategy-tower-defense": ["Strategy and tower defense", "策略塔防", "Plan defenses and manage units or resources.", "布置防线，管理单位或资源。"],
    racing: ["Racing", "竞速", "Steer through courses and compete for time.", "驾驭赛道，争取更快的成绩。"],
    runner: ["Runner", "跑酷", "Keep moving while dodging hazards and collecting rewards.", "持续前进，躲避危险并收集奖励。"],
    "roguelike-card": ["Roguelike cards", "肉鸽卡牌", "Build a deck across runs with changing encounters.", "在变化的冒险中构筑牌组。"],
    idle: ["Idle", "放置", "Accumulate progress and upgrades, including offline gains.", "积累进度与升级，支持离线收益。"],
    rhythm: ["Rhythm", "节奏", "Time actions to musical beats.", "跟随音乐节拍完成操作。"],
    arena: ["Arena", "竞技场", "Compete in short encounters inside a bounded arena.", "在限定场地中进行短局对战。"],
    "open-world": ["Open world", "开放世界", "Explore a connected world at your own pace.", "按自己的节奏探索连贯的世界。"],
  },
  view: {
    "side-2d": ["2D side view", "2D 横版", "See movement and jumps from the side.", "从侧面观察移动与跳跃。"],
    "top-down-2d": ["2D top-down", "2D 俯视", "See a flat playfield from above.", "从上方观察平面的游戏场地。"],
    "isometric-2.5d": ["Isometric", "等距 2.5D", "View a 3D world from a fixed angled camera.", "通过固定斜角镜头观察三维世界。"],
    "third-person-3d": ["3D third person", "3D 第三人称", "Follow the character from an outside camera.", "通过角色外部的镜头跟随角色。"],
    "first-person-3d": ["3D first person", "3D 第一人称", "Look through the controlled character's eyes.", "通过受控角色的双眼观察世界。"],
  },
  ui: {
    default: ["Core default", "核心默认", "The engine core UI with its built-in components.", "引擎核心界面及其内置组件。"],
    anime: ["Anime", "二次元", "Dark translucent glass, soft corners and clean sans type.", "深色半透明玻璃、柔和圆角与简洁无衬线字体。"],
    candy: ["Candy", "糖果", "Glossy plastic with a deep lip, round shapes and bouncy motion.", "带厚底边的光泽塑料、圆润造型与弹性动效。"],
    cartoon: ["Cartoon", "卡通", "Flat panels under coloured headers, thick outlines and bouncy motion.", "带彩色标题栏的平面面板、粗描边与弹性动效。"],
    fantasy: ["Fantasy", "奇幻", "Bevelled wood, round corners and a classic serif.", "带倒角的木质面板、圆角与经典衬线字体。"],
    ink: ["Ink", "水墨", "Ink on rice paper, brushed edges and Song type.", "宣纸上的水墨、笔刷边缘与宋体。"],
    minimal: ["Minimal", "极简", "Matte faces with a hairline, soft corners and calm motion.", "带细线的哑光面、柔和圆角与沉静动效。"],
    pixel: ["Pixel", "像素", "Stepped pixel edges, a pixel face and stepped motion.", "阶梯像素边缘、像素字体与逐帧动效。"],
    scifi: ["Sci-fi", "科幻", "Dark glass, four cut corners and condensed technical type.", "深色玻璃、四角切角与窄体技术字。"],
    storybook: ["Storybook", "绘本", "Illustrated paper, quiet ornament and serif type.", "插画纸面、克制装饰与衬线字体。"],
    tactical: ["Tactical", "战术", "Brushed metal, cut corners and stencil type.", "拉丝金属、切角与模版字。"],
    "tactical-arctic": ["Tactical Arctic", "极地战术", "Cold brushed steel, crisp arctic accents and cut-corner stencil readouts.", "冷调拉丝金属、极地蓝强调色与切角模版读数。"],
  },
  platform: {
    "phone-portrait": ["Phone portrait", "手机竖屏", "A tall mobile screen with touch by default.", "纵向手机画面，默认触控。"],
    "phone-landscape": ["Phone landscape", "手机横屏", "A wide mobile screen with touch by default.", "横向手机画面，默认触控。"],
    desktop: ["Desktop", "桌面", "A desktop screen with keyboard and mouse by default.", "桌面画面，默认键盘鼠标。"],
    multi: ["Multiple platforms", "多平台", "Adapt to different screens; default to keyboard and mouse.", "适应不同屏幕，默认键盘鼠标。"],
  },
  input: {
    touch: ["Touch", "触控", "Use on-screen movement and action controls.", "使用屏幕上的移动与动作控件。"],
    "keyboard-mouse": ["Keyboard and mouse", "键盘鼠标", "Use keyboard movement and mouse controls.", "使用键盘移动与鼠标操作。"],
    gamepad: ["Gamepad", "手柄", "Use controller sticks and buttons.", "使用手柄摇杆与按键。"],
  },
  players: {
    single: ["Single player", "单人", "One locally controlled player.", "一名本地控制的玩家。"],
    "local-multiplayer": ["Local multiplayer", "本地多人", "Two players share one device.", "两名玩家共用一台设备。"],
    "online-lockstep": ["Online multiplayer", "联网多人", "Players join a deterministic lockstep room.", "玩家加入确定性锁步同步房间。"],
  },
  music: {
    chiptune: ["Chiptune", "芯片音乐", "Retro synthesized tones and short melodic loops.", "复古合成音色与短旋律循环。"],
    orchestral: ["Orchestral", "管弦乐", "Strings, brass and orchestral percussion.", "弦乐、铜管与管弦打击乐。"],
    electronic: ["Electronic", "电子乐", "Synthesizers and electronic rhythms.", "合成器音色与电子节奏。"],
    ambient: ["Ambient", "氛围音乐", "Slow textures that support the scene.", "以缓慢的音色层次烘托场景。"],
    folk: ["Folk", "民谣", "Acoustic instruments and traditional melodies.", "原声乐器与传统旋律。"],
    rock: ["Rock", "摇滚", "Guitars, bass and driving drums.", "吉他、贝斯与有力的鼓点。"],
  },
} satisfies { [K in ChoiceKey]: Record<(typeof CREATION_CHOICES)[K][number], Copy> };

export function localized([en, zh, enDescription, zhDescription]: Copy) {
  return { label: { en, "zh-CN": zh }, description: { en: enDescription, "zh-CN": zhDescription } };
}

/** Chinese catalogue copy; English names and descriptions remain catalogue-owned. */
export const LOOK_ZH: Readonly<Record<string, readonly [string, string]>> = {
  "inked-comic-3d": ["墨线漫画三维", "立体模型保留粗轮廓、手绘表面与夸张色彩。"],
  "faceted-low-poly": ["低多边形切面", "可见多边形切面与大色块通过轮廓和层次呈现空间。"],
  pixel: ["像素", "严格像素网格上的精灵与瓦片、阶梯明暗与视差背景；保留俯视瓦片世界变体。"],
  "neon-vector": ["霓虹矢量", "暗色背景上的细发光线、网格与明亮粒子呈现矢量显示效果。"],
  "toy-3d": ["玩具三维", "圆润倒角的缎面塑料玩具、明亮色彩与柔和接触阴影；保留纸艺变体。"],
  "cartoon-2d": ["卡通二维", "简洁造型、每种材质两三个平涂色阶与暖深色轮廓。"],
};

/** Chinese copy of the catalogue's variants, keyed `<look>.<variant>`; English titles and descriptors remain catalogue-owned. */
export const VARIANT_ZH: Readonly<Record<string, readonly [string, string]>> = {
  "pixel.topdown": ["俯视瓦片世界", "俯视或四分之三视角的瓦片地图与直立精灵，台阶与悬崖清楚表达高度。"],
  "toy-3d.paper": ["纸艺", "折叠的平面、可见的纸张厚度与层叠剪纸构成整个世界。"],
};

/** The genre told in segments (og.flow, project.ts's `flow`): its form is the world its live scenes draw in (3d unless named), and it may leave its view out. */
export const FILM_GENRE = "interactive-film";

/** Each form's views; a game's view is one of its form's. */
export const FORM_VIEWS: { readonly [F in Form]: readonly View[] } = {
  "2d": ["side-2d", "top-down-2d"],
  "2.5d": ["isometric-2.5d"],
  "3d": ["third-person-3d", "first-person-3d"],
};

/** Each genre's views, preferred first: a game's default view is the first of its form's (none: the view is left out unless named). */
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
export function formOf(view: View): Form {
  return view.endsWith("-2d") ? "2d" : view.endsWith("-2.5d") ? "2.5d" : "3d";
}

/** The create block's fields, in its order. */
export const CREATION_FIELDS = ["form", "genre", "view", "look", "variant", "ui", "platform", "input", "players", "lang", "music"] as const;

/** A game's default view: its genre's first view of its form. */
export function defaultView(genre: Genre, form: Form, views: readonly string[] = FORM_VIEWS[form]): View | undefined {
  return GENRE_VIEWS[genre].find(v => formOf(v) === form && views.includes(v));
}

export type CreationChoices = { readonly [K in Exclude<ChoiceKey, "view">]: Choice<K> } & {
  /** Left out by an interactive film that names none: its segments frame themselves. */
  readonly view?: View;
  readonly look: string;
  /** One of the look's variants (docs/art.md §2.11); absent: the look as it is. */
  readonly variant?: string;
  readonly lang: readonly string[];
};
export type CreationInput = { -readonly [K in keyof CreationChoices]?: CreationChoices[K] };

const RULES: Partial<Record<ChoiceKey, readonly (readonly [string, RegExp])[]>> = {
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
    required: CREATION_FIELDS.filter(k => k !== "variant" && k !== "view"),
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
    // A game's view is one of its form's; every genre but interactive-film names one.
    allOf: [
      { if: { properties: { genre: { not: { const: FILM_GENRE } } } }, then: { required: ["view"] } },
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
 * one of its form's (its genre's first when nothing names one). An interactive film is form 3d unless named, and has no view unless named.
 */
export function inferCreationChoices(premise: string, input: CreationInput = {}, defaults: CreationInput = {}): CreationChoices {
  const inferred: Record<string, string> = {};
  for (const [key, rules] of Object.entries(RULES)) {
    const match = rules.find(([, pattern]) => pattern.test(premise));
    if (match) inferred[key] = match[0];
  }
  const validated = parseCreationInput(input), fallback = parseCreationInput(defaults), suggestions = parseCreationInput(inferred);
  if (validated.form && validated.view && formOf(validated.view) !== validated.form) throw new Error(`view ${validated.view} is ${formOf(validated.view)}, not form ${validated.form}; a ${validated.form} view is one of ${FORM_VIEWS[validated.form].join(", ")}`);
  const platform = validated.platform ?? suggestions.platform ?? fallback.platform ?? "desktop";
  const players = validated.players ?? suggestions.players ?? fallback.players ?? "single";
  const viewForm = (v?: View) => v && formOf(v);
  const genre = validated.genre ?? suggestions.genre ?? fallback.genre ?? "action-platformer";
  const film = genre === FILM_GENRE;
  const form: Form = validated.form ?? viewForm(validated.view) ?? (film ? "3d" : undefined) ?? suggestions.form ?? viewForm(suggestions.view) ?? fallback.form ?? viewForm(fallback.view) ?? (players === "single" ? "3d" : "2d");
  const lang = validated.lang ?? suggestions.lang ?? fallback.lang ?? (/[\u3040-\u30ff]/u.test(premise) ? ["ja"] : /[\u4e00-\u9fff]/u.test(premise) ? ["zh"] : ["en"]);
  const view = validated.view ?? (film ? undefined : [suggestions.view, fallback.view].find(v => v && formOf(v) === form) ?? defaultView(genre, form)!);
  const pick = <K extends "look" | "ui" | "music" | "input">(k: K, base: CreationChoices[K]) => validated[k] ?? suggestions[k] ?? fallback[k] ?? base;
  const variant = validated.variant ?? fallback.variant;
  return {
    form, genre, ...(view === undefined ? {} : { view }), look: pick("look", "faceted-low-poly"), ...(variant === undefined ? {} : { variant }),
    ui: pick("ui", "default"), platform,
    input: pick("input", platform.startsWith("phone-") ? "touch" : "keyboard-mouse"),
    players, lang, music: pick("music", "ambient"),
  };
}
