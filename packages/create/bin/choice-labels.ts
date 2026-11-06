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
    cartoon: ["Cartoon", "卡通", "Rounded shapes and playful controls.", "圆润的形状与活泼的控件。"],
    fantasy: ["Fantasy", "奇幻", "Ornamental panels for magical worlds.", "适合魔法世界的装饰面板。"],
    pixel: ["Pixel", "像素", "Grid-aligned controls with crisp pixel edges.", "网格对齐的控件与清晰的像素边缘。"],
    scifi: ["Science fiction", "科幻", "Technical displays and futuristic controls.", "技术显示屏与未来感控件。"],
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
  "expressionist-painting": ["表现主义绘画", "可见笔触、冲突的色块与变形的人物构成充满情绪的画面。"],
  "bold-comic-illustration": ["粗线漫画", "强烈轮廓、清晰明暗与深影中的高饱和点缀让动作一目了然。"],
  "watercolor-wash": ["水彩晕染", "透明叠色、边缘积色与纸面留白营造柔和画面。"],
  "pixel-3d-hybrid": ["像素与三维混合", "清晰的像素角色置于具有光影与景深的三维场景。"],
  "faceted-low-poly": ["低多边形切面", "可见多边形切面与大色块通过轮廓和层次呈现空间。"],
  "sculpted-clay": ["黏土雕塑", "圆钝边缘、凹凸表面与手捏痕迹呈现定格动画质感。"],
  "8bit-pixel": ["8 位像素", "小型精灵、少量色阶与严格的重复网格呈现早期主机风格。"],
  "voxel-worlds": ["体素世界", "方块构成角色、建筑与地形，保留阶梯状轮廓。"],
  "illuminated-manuscript": ["彩饰手抄本", "平面人物、墨线、装饰边框与文字组成书页式场景。"],
  "rubber-hose-animation": ["橡皮管动画", "圆头、柔软管状四肢与夸张伸缩重现早期卡通。"],
  "pencil-hatching": ["铅笔排线", "以纸面上的线条密度与方向表现灰色或棕褐色明暗。"],
  "gothic-contour": ["哥特装饰", "简洁角色置于繁复卷曲的建筑与朦胧纵深之中。"],
  "lush-painted-worlds": ["浓彩绘画世界", "厚重的不透明色块、选择性笔触与冷暖对比构成茂密环境。"],
  "decorative-storybook": ["装饰绘本", "柔和轮廓、花纹植物与温润色块构成可探索的绘本世界。"],
  "classical-painting-collage": ["古典油画拼贴", "古典绘画的人物与风景重新组合，保留油画表面和剪裁接缝。"],
  "detailed-doodle": ["精细线描世界", "细小人物与物品以略不规则的简单线条构成巨大手绘地图。"],
  "black-figure-pottery": ["黑绘陶器", "赭色底面上的深色剪影与细刻纹饰由回纹和横带围合。"],
  "heavy-ink-shadows": ["浓墨重影", "大块黑影、粗排线与低饱和色塑造变形人物，以亮部提示危险。"],
  "monochrome-pixel": ["单色像素", "黑白或单色像素依靠轮廓与留白表达，少量强调色传递信号。"],
  "16bit-pixel": ["16 位像素", "丰富精灵、阶梯明暗、视差背景与饱和色彩呈现后期主机风格。"],
  "detailed-pixel": ["精细像素插画", "丰富像素簇、细微色相变化与精细场景保留可见像素网格。"],
  "luminous-pixel": ["发光像素", "暗色底面上的饱和光源、轮廓光与辉光组织画面。"],
  "topdown-tile-pixel": ["俯视瓦片世界", "俯视瓦片地图与直立精灵通过台阶、悬崖和遮挡表现路径。"],
  "low-resolution-3d": ["低分辨率三维", "简化模型、小纹理与像素化输出呈现早期三维主机质感。"],
  "coarse-pixel": ["极简像素", "极少像素、无面孔与稀少纹理依靠轮廓和留白表达情绪。"],
  "anime-cel-3d": ["动漫赛璐珞三维", "三维模型使用二维动画的硬边阴影、轮廓线与镜头节奏。"],
  "inked-comic-3d": ["墨线漫画三维", "立体模型保留粗轮廓、手绘表面与夸张色彩。"],
  "paper-craft": ["纸艺", "折叠平面、可见纸张厚度与分层剪纸构成世界。"],
  "miniature-diorama": ["微缩景观", "高位斜角镜头、小巧圆润建筑与柔和阴影呈现桌面模型。"],
  "painterly-3d": ["绘画感三维", "简化材质、塑形与绘画配色让三维空间远离照片感。"],
  "cinematic-realism": ["电影写实", "可信材质、人体比例与自然光通过电影构图和色彩组织。"],
  "muted-cinematic-3d": ["克制舞台光", "低饱和三维形体由局部光束与薄雾照亮，以明暗层次叙事。"],
  "layered-silhouette": ["分层剪影", "实色前景轮廓与更亮、更朦胧的后景层次表现纵深。"],
  "minimal-geometric": ["极简几何", "少量颜色的矩形、圆形与三角形通过比例、节奏和留白排列。"],
  "neon-vector": ["霓虹矢量", "暗色背景上的细发光线、网格与明亮粒子呈现矢量显示效果。"],
  "monochrome-dither-3d": ["单色抖动三维", "三维场景缩减为一至两色，以点阵表现明暗过渡。"],
  "luminous-layered-painting": ["发光分层绘景", "多层横版绘景以暗色块衬托强烈发光焦点。"],
};
