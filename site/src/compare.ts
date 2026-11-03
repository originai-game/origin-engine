// The comparison's feature table. Each cell is what the engine's own
// documentation says it ships, judged strictly: "plugin" is an official,
// maintained plugin or package; "partial" is part of the feature or a
// building block for it; community add-ons count as "no", with a note.
// Sizes and frame times are in data/benchmarks.json, which the benchmark
// overwrites.
export type Support = "yes" | "plugin" | "partial" | "no";
export interface Cell { v: Support; note?: { en: string; zh: string }; source?: string }
export const engines = ["origin", "three", "babylon", "playcanvas", "unity", "godot"] as const;
export type Engine = (typeof engines)[number];

export const engineNames: Record<Engine, string> = {
  origin: "Origin Engine",
  three: "three.js",
  babylon: "Babylon.js",
  playcanvas: "PlayCanvas",
  unity: "Unity Web",
  godot: "Godot Web",
};
export const versions: Record<Engine, string> = {
  origin: "0.7.7",
  three: "r187",
  babylon: "9.x",
  playcanvas: "2.23",
  unity: "6.x",
  godot: "4.5",
};

const n = (en: string, zh: string) => ({ en, zh });
const c = (v: Support, note?: { en: string; zh: string }, source?: string): Cell => ({ v, ...(note ? { note } : {}), ...(source ? { source } : {}) });

export const features: { key: string; label: { en: string; zh: string }; cells: Record<Engine, Cell> }[] = [
  {
    key: "probe",
    label: n("Agent probe and headless runs: world state as JSON, frames, scripted input", "Agent probe 与无头运行：JSON 世界状态、截帧、脚本输入"),
    cells: {
      origin: c("yes", n("probe protocol, og run --headless --commands", "probe 协议，og run --headless --commands")),
      three: c("no"),
      babylon: c("partial", n("NullEngine runs headless; no state or frame probe", "NullEngine 可无头运行；无状态与截帧 probe")),
      playcanvas: c("partial", n("NullGraphicsDevice runs headless; no probe", "NullGraphicsDevice 可无头运行；无 probe")),
      unity: c("partial", n("batch mode and the Test Framework; no runtime probe", "批处理模式与测试框架；无运行时 probe")),
      godot: c("partial", n("--headless; no runtime probe", "--headless；无运行时 probe")),
    },
  },
  {
    key: "determinism",
    label: n("Deterministic replay and rollback multiplayer", "确定性回放与回滚联机"),
    cells: {
      origin: c("yes", n("fixed tick, engine-owned RNG, whole-world snapshots", "固定 tick、引擎掌管的随机数、整世界快照")),
      three: c("no"),
      babylon: c("partial", n("deterministic physics stepping; no rollback", "物理可确定步进；无回滚")),
      playcanvas: c("no"),
      unity: c("plugin", n("Netcode for Entities (prediction and rollback)", "Netcode for Entities（预测与回滚）")),
      godot: c("no", n("community add-ons only", "仅有社区插件")),
    },
  },
  {
    key: "physics",
    label: n("Physics", "物理"),
    cells: {
      origin: c("yes"),
      three: c("no", n("third-party libraries", "第三方库")),
      babylon: c("plugin", n("Havok plugin", "Havok 插件")),
      playcanvas: c("plugin", n("ammo.js integration", "ammo.js 集成")),
      unity: c("yes"),
      godot: c("yes"),
    },
  },
  {
    key: "navigation",
    label: n("Navigation meshes and crowds", "导航网格与群体寻路"),
    cells: {
      origin: c("yes", n("og.nav, og.Crowd", "og.nav、og.Crowd")),
      three: c("no", n("third-party libraries", "第三方库")),
      babylon: c("plugin", n("Recast plugin", "Recast 插件")),
      playcanvas: c("no", n("third-party libraries", "第三方库")),
      unity: c("yes", n("AI Navigation package", "AI Navigation 包")),
      godot: c("yes"),
    },
  },
  {
    key: "ui",
    label: n("Game UI system: HUD widgets, layout, localisation", "游戏 UI 系统：HUD 控件、布局、本地化"),
    cells: {
      origin: c("yes", n("17 genre screens, 12 style packs, RTL and CJK", "17 个品类界面、12 套风格包、RTL 与 CJK")),
      three: c("no"),
      babylon: c("yes", n("Babylon GUI", "Babylon GUI")),
      playcanvas: c("yes", n("Screen and Element components", "Screen 与 Element 组件")),
      unity: c("yes", n("UI Toolkit, uGUI", "UI Toolkit、uGUI")),
      godot: c("yes", n("Control nodes", "Control 节点")),
    },
  },
  {
    key: "meshlets",
    label: n("Meshlet rendering with automatic LOD and impostors", "Meshlet 渲染与自动 LOD、impostor"),
    cells: {
      origin: c("yes"),
      three: c("no", n("LOD by hand", "手动 LOD")),
      babylon: c("no", n("LOD by hand", "手动 LOD")),
      playcanvas: c("no"),
      unity: c("no", n("LOD groups, authored", "LOD Group，需手工制作")),
      godot: c("partial", n("automatic mesh LOD; no meshlets", "自动网格 LOD；无 meshlet")),
    },
  },
  {
    key: "gi",
    label: n("Global illumination in the web build", "Web 构建中的全局光照"),
    cells: {
      origin: c("yes", n("baked GI, reflection probes, volumetric light", "烘焙 GI、反射探针、体积光")),
      three: c("partial", n("lightmaps baked elsewhere", "使用外部烘焙的光照贴图")),
      babylon: c("partial", n("lightmaps baked elsewhere, IBL", "外部烘焙光照贴图、IBL")),
      playcanvas: c("yes", n("built-in lightmapper", "内置光照烘焙")),
      unity: c("yes", n("baked lightmaps", "烘焙光照贴图")),
      godot: c("partial", n("LightmapGI; no SDFGI or VoxelGI on the web", "LightmapGI；Web 端无 SDFGI、VoxelGI")),
    },
  },
  {
    key: "webgpu",
    label: n("WebGPU renderer", "WebGPU 渲染器"),
    cells: {
      origin: c("yes"),
      three: c("yes", n("WebGPURenderer", "WebGPURenderer")),
      babylon: c("yes"),
      playcanvas: c("yes"),
      unity: c("partial", n("experimental", "实验性")),
      godot: c("no", n("WebGL 2 (Compatibility renderer)", "WebGL 2（兼容渲染器）")),
    },
  },
  {
    key: "typescript",
    label: n("Game code in TypeScript, one API on web and native", "游戏代码用 TypeScript，Web 与原生同一套 API"),
    cells: {
      origin: c("yes"),
      three: c("partial", n("web only; types from DefinitelyTyped", "仅 Web；类型来自 DefinitelyTyped")),
      babylon: c("partial", n("web only", "仅 Web")),
      playcanvas: c("partial", n("web only", "仅 Web")),
      unity: c("no", n("C#", "C#")),
      godot: c("no", n("GDScript, C#", "GDScript、C#")),
    },
  },
];
