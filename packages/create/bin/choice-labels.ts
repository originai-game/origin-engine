import type { CreationChoices } from "./choices.ts";

type Copy = readonly [enLabel: string, zhLabel: string, enDescription: string, zhDescription: string];
type EnumField = Exclude<keyof CreationChoices, "look" | "lang">;

/** Form copy keyed by stable IDs; adding a choice requires both translations. */
export const CHOICE_LABELS = {
  genre: {
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
  mood: {
    bright: ["Bright", "明亮", "Clear light and lively colour.", "清晰的光线与活泼的色彩。"],
    dark: ["Dark", "阴暗", "Deep shadows and a tense atmosphere.", "深沉的阴影与紧张的氛围。"],
    warm: ["Warm", "温暖", "Warm light and welcoming colour.", "暖色光线与亲切的配色。"],
    cold: ["Cold", "冷峻", "Cool light and a distant atmosphere.", "冷色光线与疏离的氛围。"],
    soft: ["Soft", "柔和", "Gentle contrast and restrained light.", "温和的对比与克制的光线。"],
    "high-contrast": ["High contrast", "高对比", "Strong separation between light and shadow.", "鲜明区分明部与暗部。"],
  },
  ui: {
    default: ["Core default", "核心默认", "The engine core UI with its built-in components.", "引擎核心界面及其内置组件。"],
    cartoon: ["Cartoon", "卡通", "Rounded shapes, bold colour and playful controls.", "圆润的形状、鲜明的色彩与活泼的控件。"],
    fantasy: ["Fantasy", "奇幻", "Warm textured panels and ornament for magical worlds.", "适合魔法世界的温暖质感面板与装饰。"],
    scifi: ["Science fiction", "科幻", "Dark translucent glass and technical displays.", "深色半透明玻璃与技术显示屏。"],
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
} satisfies { [K in EnumField]: Record<CreationChoices[K], Copy> };

export function localized([en, zh, enDescription, zhDescription]: Copy) {
  return { label: { en, "zh-CN": zh }, description: { en: enDescription, "zh-CN": zhDescription } };
}

/** Chinese catalogue copy; English names and descriptions remain catalogue-owned. */
export const LOOK_ZH: Readonly<Record<string, readonly [string, string]>> = {
  "cinematic-realism": ["电影写实", "可信材质、人体比例与自然光通过电影构图和色彩组织。"],
  "anime-cel-3d": ["动漫赛璐珞三维", "三维模型使用二维动画的硬边阴影、轮廓线与镜头节奏。"],
  "inked-comic-3d": ["墨线漫画三维", "立体模型保留粗轮廓、手绘表面与夸张色彩。"],
  "painterly-3d": ["绘画感三维", "简化材质、塑形与绘画配色让三维空间远离照片感。"],
  "faceted-low-poly": ["低多边形切面", "可见多边形切面与大色块通过轮廓和层次呈现空间。"],
  "low-resolution-3d": ["低分辨率三维", "简化模型、小纹理与像素化输出呈现早期三维主机质感。"],
  pixel: ["像素", "严格像素网格上的精灵与瓦片、阶梯明暗与视差背景；变体涵盖 8 位、单色、精细与发光像素。"],
  "minimal-geometric": ["极简几何", "少量颜色的矩形、圆形与三角形通过比例、节奏和留白排列。"],
  "neon-vector": ["霓虹矢量", "暗色背景上的细发光线、网格与明亮粒子呈现矢量显示效果。"],
  "toy-3d": ["玩具三维", "圆润倒角的缎面塑料玩具、明亮色彩与柔和接触阴影；变体涵盖体素、黏土、微缩景观与纸艺。"],
  "cartoon-2d": ["卡通二维", "简洁造型、每种材质两三个平涂色阶与暖深色轮廓；变体涵盖水彩、绘本与橡皮管动画。"],
  "pixel-3d-hybrid": ["像素与三维混合", "清晰的像素角色置于具有光影与景深的三维场景。"],
};
