// Every word on the page, in English and Chinese. Chinese is written, not
// translated word for word: it follows the design language's CJK rules
// (zero tracking, looser leading) and keeps code, commands and names in Latin.
export type Lang = "en" | "zh";
export const langs: Lang[] = ["en", "zh"];
export const path = (lang: Lang) => (lang === "en" ? "/" : "/zh/");

type T = { en: string; zh: string };
const t = (en: string, zh: string): T => ({ en, zh });

export const copy = {
  meta: {
    title: t("Origin Engine — the game engine built for AI", "Origin Engine — 为 AI 而生的游戏引擎"),
    description: t(
      "A WebGPU game engine designed to be driven by AI agents: one TypeScript API, a probe that answers in JSON, deterministic to the bit, rendered live in the browser.",
      "为 AI agent 设计的 WebGPU 游戏引擎：一套 TypeScript API，用 JSON 回答的 probe，逐位确定，在浏览器中实时渲染。",
    ),
  },
  nav: {
    loop: t("AI-native", "AI 原生"),
    rendering: t("Rendering", "渲染"),
    characters: t("Characters", "角色"),
    worlds: t("Worlds", "世界"),
    ui: t("UI", "UI"),
    play: t("Play", "试玩"),
    switchTo: t("中文", "EN"),
    switchLabel: t("阅读中文版", "Read in English"),
  },
  placeholder: {
    pending: t("Placeholder", "占位"),
    interim: t("Interim", "临时素材"),
    interimNote: t("Real engine output standing in until the new footage lands", "真实引擎画面，新素材到位前临时使用"),
  },
  hero: {
    title: t("The game engine built for AI.", "为 AI 而生的游戏引擎。"),
    live: t("Every frame of this scene is rendered live in your browser, by the engine itself.", "这一幕由引擎在你的浏览器里实时渲染，每一帧都是。"),
    recorded: t("Recorded from the engine. Open this page in a browser with WebGPU to watch it live.", "引擎实录画面。用支持 WebGPU 的浏览器打开，即可观看实时渲染。"),
    captured: t("A frame captured from the engine. The live scene starts in browsers with WebGPU.", "引擎截取的画面。在支持 WebGPU 的浏览器中会播放实时场景。"),
    badgeLive: t("Live · WebGPU · not a video", "实时 · WebGPU · 非视频"),
    badgeVideo: t("Engine recording", "引擎录像"),
    badgeFrame: t("Captured frame", "引擎截帧"),
    badgeLoading: t("Starting the engine", "引擎启动中"),
    interim: t("Interim frame: the live ocean scene is in production", "临时画面：实时海天场景制作中"),
    scroll: t("Scroll", "向下"),
  },
  loop: {
    eyebrow: t("AI-native", "AI 原生"),
    title: t("It writes the game. Then it watches it run.", "它写游戏，也看着游戏运行。"),
    body: t(
      "Origin Engine was designed to be driven by an agent: one TypeScript API on every host, a probe that answers in JSON, and frames the agent reads like a player would. This is a real session, replayed. The agent describes the frame, a test fails, it finds why, and the fix goes green.",
      "Origin Engine 从第一天起就是给 agent 驱动的：所有平台同一套 TypeScript API，probe 用 JSON 回答问题，画面 agent 能像玩家一样读懂。下面是一次真实会话的回放：agent 描述画面，测试失败，找到原因，修复后转绿。",
    ),
    steps: [t("Describe", "描述"), t("Test", "测试"), t("Diagnose", "诊断"), t("Fix", "修复")],
    agentSays: t("The crate never fell: Drop is missing from systems.", "箱子没有掉下来：systems 里少了 Drop。"),
    frameBroken: t("Tick 120 · broken", "第 120 帧 · 出错"),
    frameFixed: t("Tick 120 · fixed", "第 120 帧 · 修复后"),
    frameFirst: t("Tick 1 · described", "第 1 帧 · 已描述"),
    provenance: t(
      "Replay of the probe-depth acceptance session: commands and output as recorded, frames from its CPU renderer.",
      "回放 probe-depth 验收会话：命令与输出均为原始记录，画面来自其 CPU 渲染。",
    ),
    replay: t("Replay", "重播"),
    tools: t(
      "The same probe answers the inspector, the gizmos and every og command: describe, pick, expect, capture, trace, dump, save the scene back to text.",
      "检查器、gizmo 和每一条 og 命令用的都是同一个 probe：describe、pick、expect、capture、trace、dump，以及把场景写回文本。",
    ),
  },
  rendering: {
    eyebrow: t("Rendering", "渲染"),
    title: t("Real light, in a browser tab.", "浏览器标签页里的真实光照。"),
    body: t(
      "WebGPU in the browser and Metal natively: a physical sky with volumetric clouds, an open ocean, weather that wets and snows the world, baked global illumination, reflection probes, meshlets with LOD and impostors, TAA upscaling. Drag across a frame to see what the engine sees.",
      "浏览器里用 WebGPU，原生用 Metal：物理天空与体积云、开阔海洋、会让世界变湿和积雪的天气、烘焙全局光照、反射探针、带 LOD 和 impostor 的 meshlet、TAA 超分。在画面上拖动，看看引擎眼中的世界。",
    ),
    final: t("Final", "成片"),
    engineView: t("Engine view", "引擎视图"),
    off: t("Off", "关"),
    on: t("On", "开"),
    before: t("Before", "之前"),
    after: t("After", "之后"),
    wipes: {
      fog: t("Volumetric light", "体积光"),
      refl: t("Reflection probes", "反射探针"),
      ssr: t("Screen-space reflections", "屏幕空间反射"),
    },
    weatherTitle: t("One avenue, one camera, seven skies.", "同一条大道，同一机位，七种天气。"),
    weather: {
      fair: t("Fair", "晴"),
      sunset: t("Sunset", "日落"),
      overcast: t("Overcast", "阴"),
      rain: t("Rain", "雨"),
      lightning: t("Storm", "雷暴"),
      snow: t("Snow", "积雪"),
      dust: t("Dust", "沙尘"),
    },
    oceanTitle: t("An open sea that is also physics.", "海面既是画面，也是物理。"),
    oceanBody: t(
      "The waves you see are the waves bodies float on: the renderer and og.physics read the same surface.",
      "你看到的波浪，就是物体漂浮的波浪：渲染器与 og.physics 读取同一张海面。",
    ),
  },
  characters: {
    eyebrow: t("Characters", "角色"),
    title: t("Drawn once. Rigged and moving.", "画一次，就能绑骨、动起来。"),
    body: t(
      "og.motion, a motion model trained for the engine, animates characters in real time. Cloth drapes and blows in the wind, crowds steer around each other, and a layered 2D drawing becomes a rigged character with ten clips and no hand edits.",
      "og.motion 是为引擎训练的动作模型，实时驱动角色。布料随风垂摆，人群彼此避让；一张分层的 2D 原画，不经手工调整，就变成带骨骼和十段动作的角色。",
    ),
    autorig: t("2D auto-rig", "2D 自动绑骨"),
    autorigBody: t("The knight and the mage, each from one layered drawing: idle, walk, run, jump, attack, wave, die and more.", "骑士和法师各来自一张分层原画：待机、走、跑、跳、攻击、挥手、倒下等。"),
    motion: t("og.motion", "og.motion"),
    motionBody: t("Learned motion, run on the GPU every frame.", "学习得到的动作，每帧在 GPU 上运行。"),
    cloth: t("Cloth", "布料"),
    clothBody: t("Simulated in the tick, deterministic like everything else.", "在 tick 内模拟，与其他一切一样确定。"),
  },
  worlds: {
    eyebrow: t("Worlds", "世界"),
    title: t("Worlds bigger than the screen.", "比屏幕大得多的世界。"),
    body: t(
      "Terrain with layers, holes and grass in the wind. Cells that stream in and out around the player under a memory budget. Procedural generation from a seed, the same world every time: noise terrain, BSP dungeons and wave-function-collapse villages, regenerated live.",
      "带分层、挖洞与随风草地的地形；在内存预算内围绕玩家流式加载与卸载的世界分块；由种子生成、每次都一样的程序化世界：噪声地形、BSP 地牢、WFC 村庄，实时重生。",
    ),
    terrain: t("Noise terrain", "噪声地形"),
    dungeon: t("BSP dungeons", "BSP 地牢"),
    village: t("WFC villages", "WFC 村庄"),
  },
  ui: {
    eyebrow: t("UI", "UI"),
    title: t("A UI system for every genre.", "覆盖所有品类的 UI 系统。"),
    body: t(
      "HUDs for seventeen genres, style packs that restyle every widget, every locale from right-to-left Arabic to Japanese line breaking, phone to ultrawide. Built into the engine and drawn on the GPU, with an accessibility tree behind it.",
      "十七个品类的 HUD，可重塑每个控件的风格包，从右到左的阿拉伯语到日文禁则换行的全部语言，从手机到超宽屏。内置于引擎、由 GPU 绘制，并带有无障碍树。",
    ),
    pending: t("The genre × style × language grid arrives with the seventeen genre showcases.", "品类 × 风格 × 语言的交互网格，将随十七个品类展示一起上线。"),
  },
  determinism: {
    title: t("Same inputs. Same world. Bit for bit.", "同样的输入，同样的世界，一位不差。"),
    body: t(
      "A fixed tick, randomness owned by the engine, whole-world snapshots. That is what makes replays, saves that load on any host, and rollback multiplayer hold.",
      "固定 tick、由引擎掌管的随机数、整个世界的快照。回放、在任何平台都能读取的存档、回滚联机，都建立在这之上。",
    ),
    rollback: t("Rollback, two peers", "回滚联机，双端"),
    rollbackBody: t("80 ms latency, 30 ms jitter, 5 % input loss, 336 rollbacks, 600 ticks.", "80 ms 延迟、30 ms 抖动、5% 输入丢失，336 次回滚，600 个 tick。"),
    lockstep: t("Lockstep, same session", "同一会话，锁步"),
    lockstepBody: t("The confirmed rollback world ends on the lockstep hash.", "回滚确认后的世界，与锁步结果哈希相同。"),
    save: t("A save, native and web", "同一存档，原生与网页"),
    saveBody: t("Restored on both hosts at tick 90, 144 component types.", "在两个平台上于第 90 tick 恢复，144 种组件。"),
    match: t("Identical", "完全一致"),
  },
  play: {
    eyebrow: t("Play", "试玩"),
    title: t("Games an agent made with it.", "agent 用它做出的游戏。"),
    body: t("Each one written by an agent from the engine's knowledge package alone: its skills, the wiki, the generated types and og. What the agent found missing went back into the engine.", "每一款都由 agent 只凭引擎的知识包写成：技能、wiki、生成的类型和 og。agent 发现的缺口，都回到了引擎里。"),
    soon: t("Playable build coming", "可玩版本即将上线"),
    close: t("Close", "关闭"),
    games: {
      meridian: t("A night time trial on a neon circuit, with a ghost and replays.", "霓虹赛道上的夜间计时赛，带幽灵车与回放。"),
      hollowbell: t("A 2D pixel-art platformer down the old well.", "深入古井的 2D 像素平台跳跃。"),
      brasswick: t("A puzzle-platformer in a lamplit clockwork vault.", "灯火中的发条地窖解谜平台跳跃。"),
      kestrel: t("A third-person action-adventure with cloth, rope and navmesh minds.", "带布料、绳索与导航 AI 的第三人称动作冒险。"),
      lantern: t("Light four lanterns before night falls on the valley.", "在夜色降临山谷前点亮四盏灯。"),
      rumpus: t("A party game for two to four on one screen.", "二到四人同屏的派对游戏。"),
    },
  },
  numbers: {
    eyebrow: t("Numbers", "数字"),
    title: t("Measured, not claimed.", "实测，而非宣称。"),
    body: t("Every figure is counted from the repository at this build or read from a recorded measurement. Hover one for its source.", "每个数字都来自本次构建时对仓库的统计，或一次有记录的实测。悬停可查看来源。"),
    benchmark: t("Against three.js and other web engines", "与 three.js 等 Web 引擎对比"),
    benchmarkBody: t("Head-to-head scenes are being measured. The results will appear here as they are recorded.", "同场景对比测试正在进行，结果记录后会在这里公布。"),
    pending: t("Being measured", "测量中"),
  },
  cta: {
    title: t("Make your game with an agent.", "和 AI 一起做你的游戏。"),
    body: t("Origin is where an agent builds, tests and publishes games on this engine.", "在 Origin 上，agent 用这个引擎制作、测试并发布游戏。"),
    primary: t("Start on Origin", "前往 Origin"),
    secondary: t("Create a project", "创建项目"),
    command: t("Or start from a template:", "或从模板开始："),
    copied: t("Copied", "已复制"),
    copy: t("Copy", "复制"),
  },
  footer: {
    rights: t("© 2026 Origin AI", "© 2026 Origin AI"),
    built: t("This page's numbers and media were produced by the engine at commit", "本页数字与素材由引擎产出，对应提交"),
  },
  media: {
    source: t("Source", "来源"),
    play: t("Play", "播放"),
    pause: t("Pause", "暂停"),
  },
};

export const pick = (v: T, lang: Lang) => v[lang];
