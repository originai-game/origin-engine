import type { CREATION_CHOICES } from "./choices.ts";

type Copy = readonly [enLabel: string, zhLabel: string, enDescription: string, zhDescription: string];
type EnumField = Exclude<keyof typeof CREATION_CHOICES, never>;

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
} satisfies { [K in EnumField]: Record<(typeof CREATION_CHOICES)[K][number], Copy> };

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

/** Chinese copy of the catalogue's variants, keyed `<look>.<variant>`; English titles and descriptors remain catalogue-owned. */
export const VARIANT_ZH: Readonly<Record<string, readonly [string, string]>> = {
  "cinematic-realism.muted": ["克制舞台光", "简化的三维造型与克制色彩，由舞台式的局部光束与薄雾照亮。"],
  "inked-comic-3d.bold-comic": ["醒目漫画", "粗轮廓、分组明确的明暗与深色阴影前的饱和点缀，运动中也一目了然。"],
  "inked-comic-3d.heavy-ink": ["浓墨重影", "实黑阴影、粗糙排线与低饱和色彩塑造变形的人物，高光只留给危险。"],
  "painterly-3d.lush": ["厚重油彩", "厚重不透明的色块、有选择的笔触与丰富的冷暖对比构成浓密的绘画场景。"],
  "painterly-3d.luminous-layers": ["光影层叠", "多层前景与背景的侧视绘画场景，暗部之中有强烈的发光焦点。"],
  "painterly-3d.expressionist": ["表现主义", "可见笔触、冲撞的色块与变形的面孔，让每一帧都像情绪化的画作。"],
  "low-resolution-3d.monochrome-dither": ["单色抖动三维", "三维场景简化为一两种颜色，由点阵承担明暗过渡，透视依然真实。"],
  "pixel.8bit": ["早期主机像素", "小巧的精灵、每个色阶寥寥几步硬过渡与严格网格上重复的瓦片。"],
  "pixel.monochrome": ["单墨像素", "黑白（或一墨一纸）像素，由剪影与留白表达，至多一种强调色。"],
  "pixel.detailed": ["精细插画像素", "接近插画的像素：丰富的色簇、细微的色相变化与精细场景。"],
  "pixel.luminous": ["暗底发光像素", "以暗色为底，饱和的光源、轮廓高光与辉光构成画面。"],
  "pixel.coarse": ["极简粗像素", "每个形象只有很少的像素，以剪影、留白与少量色块讲述氛围与故事。"],
  "pixel.topdown": ["俯视瓦片世界", "俯视或四分之三视角的瓦片地图与直立精灵，台阶与悬崖清楚表达高度。"],
  "minimal-geometric.layered-silhouette": ["层叠剪影", "实色前景轮廓叠在更浅、更朦胧的远景前，用形状与明度表现纵深。"],
  "toy-3d.voxel": ["体素世界", "立方体搭建角色、建筑与地形，阶梯状轮廓清晰可见。"],
  "toy-3d.clay": ["黏土雕塑", "略带凹凸的触感表面、圆钝边缘与指纹痕迹，常伴随定格动画的顿挫。"],
  "toy-3d.diorama": ["微缩景观", "高角度（常为等距）取景、小巧圆润的建筑与柔和阴影，像桌面模型。"],
  "toy-3d.paper": ["纸艺", "折叠的平面、可见的纸张厚度与层叠剪纸构成整个世界。"],
  "cartoon-2d.watercolor": ["水彩晕染", "透明的色层、边缘积聚的颜料与留白的纸面，让画面轻盈透气。"],
  "cartoon-2d.storybook": ["绘本装饰", "温和的轮廓、图案化的植物与柔和色块，组成可探索的绘本世界。"],
  "cartoon-2d.rubber-hose": ["橡皮管动画", "圆头角色、可弯曲的管状四肢与持续的挤压拉伸，墨线如早期影院动画。"],
};
