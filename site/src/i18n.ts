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
    ai: t("AI-native", "AI 原生"),
    rendering: t("Rendering", "渲染"),
    characters: t("Characters", "角色"),
    ui: t("UI", "UI"),
    compare: t("Compare", "对比"),
    switchTo: t("中文", "EN"),
    switchLabel: t("阅读中文版", "Read in English"),
  },
  placeholder: {
    pending: t("Placeholder", "占位"),
    interim: t("Interim", "临时素材"),
    interimNote: t("Real engine output standing in until the new footage lands", "真实引擎画面，新素材到位前临时使用"),
    estimate: t("Estimate — benchmark pending", "预估 — 基准测试进行中"),
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
    worldsTitle: t("Worlds bigger than the screen.", "比屏幕大得多的世界。"),
    worldsBody: t(
      "Terrain with layers and grass in the wind, cells that stream around the player under a memory budget, and procedural worlds from a seed: noise terrain, BSP dungeons and wave-function-collapse villages, the same world every time.",
      "带分层与随风草地的地形，在内存预算内围绕玩家流式加载的世界分块，以及由种子生成、每次都一样的程序化世界：噪声地形、BSP 地牢、WFC 村庄。",
    ),
    terrain: t("Noise terrain", "噪声地形"),
    dungeon: t("BSP dungeons", "BSP 地牢"),
    village: t("WFC villages", "WFC 村庄"),
    oceanBody: t(
      "The waves you see are the waves bodies float on: the renderer and og.physics read the same surface.",
      "你看到的波浪，就是物体漂浮的波浪：渲染器与 og.physics 读取同一张海面。",
    ),
  },
  characters: {
    eyebrow: t("Characters", "角色"),
    title: t("Take the controls. Watch the model move.", "亲手操控，看模型如何驱动角色。"),
    body: t(
      "og.motion is a motion model trained for the engine. Steer the character with the keyboard, mouse or touch: each step the model reads your goal, chooses a motion, decodes the pose on the GPU, plants the feet and retargets it to the skeleton, and the log beside it shows every step.",
      "og.motion 是为引擎训练的动作模型。用键盘、鼠标或触屏操控角色：每一步，模型读取你的目标、选择动作、在 GPU 上解码姿态、落脚并重定向到骨骼，旁边的日志实时显示每一步。",
    ),
    take: t("Take control", "开始操控"),
    release: t("Press Esc to give control back to the page", "按 Esc 把控制交还页面"),
    pendingLive: t("The interactive scene is being built", "交互场景制作中"),
    logs: [t("goal", "目标"), t("chosen motion", "选定动作"), t("decode time", "解码耗时"), t("foot contacts", "脚步接触"), t("retarget", "重定向")],
    autorig: t("2D auto-rig", "2D 自动绑骨"),
    autorigBody: t("A knight from one layered drawing, rigged with no hand edits: idle, walk, run, jump, attack, wave, die and more.", "一张分层原画变成的骑士，无需手工调整即完成绑骨：待机、走、跑、跳、攻击、挥手、倒下等。"),
    course: t("og.motion on the course", "训练场上的 og.motion"),
    courseBody: t("Agents driven by the model, every frame on the GPU.", "由模型驱动的角色，每帧在 GPU 上运行。"),
    cloth: t("Cloth", "布料"),
    clothBody: t("Simulated in the tick, deterministic like everything else.", "在 tick 内模拟，与其他一切一样确定。"),
  },
  ui: {
    eyebrow: t("UI", "UI"),
    title: t("A UI system that ships every genre.", "一套 UI 系统，覆盖所有品类。"),
    body: t(
      "Seventeen genre screens built into the engine, twelve style packs that re-dress every widget, every locale from Japanese to right-to-left Arabic, and layouts that hold from a phone to an ultrawide. Drawn on the GPU with an accessibility tree behind it. Switch them yourself: every frame below is the engine's own.",
      "引擎内置十七个品类的界面、可重塑每个控件的十二套风格包、从日文到从右到左阿拉伯语的全部语言，布局从手机到超宽屏都成立。由 GPU 绘制，背后有无障碍树。自己切换看看：下面每一帧都是引擎自己画的。",
    ),
    genres: t("Genres", "品类"),
    packs: t("Style packs", "风格包"),
    locale: t("Language", "语言"),
    device: t("Device", "设备"),
    screen: t("Screen", "界面"),
    packNote: t("Pack frames are in English; U8's showcases bring every pack in every locale.", "风格包画面暂为英文；U8 展示集上线后，每套风格包都将覆盖全部语言。"),
  },
  compare: {
    eyebrow: t("Compare", "对比"),
    title: t("What you get without writing it yourself.", "不必自己写，就已经有了。"),
    body: t(
      "Web engines are good at drawing. Origin Engine ships the rest of the game runtime too, and makes it readable by an agent. Where another engine gets there through an official plugin, or by hand, the table says so.",
      "Web 引擎擅长绘制。Origin Engine 把游戏运行时的其余部分也一并提供，并且让 agent 读得懂。其他引擎通过官方插件或手写代码能做到的，表格里如实标注。",
    ),
    feature: t("Feature", "功能"),
    yes: t("Built in", "内置"),
    plugin: t("Official plugin", "官方插件"),
    partial: t("Partial", "部分"),
    no: t("No", "无"),
    perfTitle: t("The same scene, frame for frame.", "同一场景，逐帧对比。"),
    perfBody: t(
      "5,261 props (3.0 M triangles at full detail), 48 skinned characters, cascaded and local shadows, AO, bloom and TAA, at 1280×720. Both sides load the same files. three.js hand-instanced on WebGL2 comes within 2×; with InstancedMesh written by hand it can close more of the gap. Our point is that the engine does it by default.",
      "5,261 个物件（满细节 300 万三角形）、48 个蒙皮角色、级联与局部阴影、AO、泛光与 TAA，1280×720。双方加载同样的文件。three.js 在 WebGL2 上手写实例化后差距在 2 倍以内；手写 InstancedMesh 还能进一步缩小差距。我们的主张是：引擎默认就做到了。",
    ),
    perfUnit: t("ms per frame, lower is better", "每帧毫秒，越低越好"),
    sizeTitle: t("What a player downloads.", "玩家需要下载多少。"),
    sizeBody: t(
      "Origin Engine's figure is the whole 3D runtime: renderer, physics, UI, audio and navigation. A rendering library's figure is the renderer alone; physics and UI come on top.",
      "Origin Engine 的数字是完整的 3D 运行时：渲染、物理、UI、音频与导航。渲染库的数字只是渲染器本身，物理和 UI 还要另加。",
    ),
    measured: t("Measured", "实测"),
    basis: t("Basis", "依据"),
    numbersTitle: t("The engine, measured.", "引擎实测。"),
    pending: t("Being measured", "测量中"),
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


  cta: {
    eyebrow: t("Start", "开始"),
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
