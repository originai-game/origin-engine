// Every picture and film the site shows, by slot. A source is a showcase
// archive entry (`showcase/manifest.jsonl`, fetched from the bucket), a file
// committed in this repository (a reference frame, evidence), or `pending`:
// footage still being produced, drawn as a marked placeholder. `scripts/media.ts`
// resolves every source into `public/m/` and writes `src/generated/media.json`;
// `bun scripts/check.ts --final` refuses a build that still has a pending slot.
//
// `status` says how the slot stands against the plan: `final` is the footage
// the page is meant to carry, `interim` is real engine output standing in
// until the scenes thread's footage lands (labelled as such on the page).

export type Source =
  | { showcase: string }
  | { repo: string; commit?: string }
  | { pending: string };

export interface Slot {
  source: Source;
  status: "final" | "interim" | "pending";
  /** What the frame shows, for alt text and the provenance line. */
  alt: { en: string; zh: string };
  /** A still's crop in source pixels (ImageMagick geometry `WxH+X+Y`): drops overlays the capture burned in, such as a station title. */
  crop?: string;
  /** A film's poster comes from this second of the film. */
  posterAt?: number;
}

const s = (id: string): Source => ({ showcase: id });
const r = (path: string): Source => ({ repo: path });
const p = (what: string): Source => ({ pending: what });
const ref = (name: string): Source => r(`crates/og-tests/references/metal/${name}.png`);
// The probe-depth frames without the template's menu panel (top left).
const LOOP_CROP = "1024x576+256+30";
// fx-showcase station frames without the station title (top left) and the og.ui badge (bottom right).
const STATION_CROP = "1024x576+128+72";

export const media = {
  // ── Hero ──────────────────────────────────────────────────────────────
  // The stand-in: site-ocean, a single scene, until site-hero boots (live.ts).
  "hero-ocean-poster": {
    source: ref("water-ocean-sunset"),
    status: "interim",
    alt: { en: "The open ocean at sunset", zh: "日落时的开阔海面" },
  },
  "hero-ocean-film": {
    source: p("site-hero feature: site-ocean 90 s loop, 4K and 1080p (studio-win)"),
    status: "pending",
    alt: { en: "The ocean scene from golden hour through a storm to night", zh: "海天场景，从黄昏经风暴到夜晚" },
  },
  // site-hero's scenes: each poster is the scene's t=0 frame, its first live frame after a switch.
  "hero-skyreach-meadow-poster": {
    source: p("site-hero feature: Skyreach Meadow at t=0, 4K"),
    status: "pending",
    alt: { en: "A traveller on a cliff meadow above a sea of clouds and floating islands", zh: "云海与浮岛之上的悬崖草甸，一位旅人" },
  },
  "hero-skyreach-meadow-film": {
    source: p("site-hero feature: Skyreach Meadow loop, 4K and 1080p (studio-win)"),
    status: "pending",
    alt: { en: "Skyreach Meadow, live in the engine", zh: "天穹草甸，引擎实时画面" },
  },
  "hero-lantern-shrine-poster": {
    source: p("site-hero feature: Lantern Shrine at t=0, 4K"),
    status: "pending",
    alt: { en: "A shrine of lanterns at dusk", zh: "暮色中的灯笼神社" },
  },
  "hero-lantern-shrine-film": {
    source: p("site-hero feature: Lantern Shrine loop, 4K and 1080p (studio-win)"),
    status: "pending",
    alt: { en: "Lantern Shrine, live in the engine", zh: "灯笼神社，引擎实时画面" },
  },

  // ── AI loop ───────────────────────────────────────────────────────────
  "loop-first": {
    source: r("tests/evidence/depth/probe-depth/first-frame.png"),
    status: "interim",
    crop: LOOP_CROP,
    alt: { en: "The template game at tick 1, as the probe captured it", zh: "模板游戏第 1 帧，probe 截取" },
  },
  "loop-broken": {
    source: r("tests/evidence/depth/probe-depth/broken-tick120.png"),
    status: "interim",
    crop: LOOP_CROP,
    alt: { en: "The broken copy at tick 120: the crate still hangs in the air", zh: "出错的副本第 120 帧：箱子仍悬在空中" },
  },
  "loop-fixed": {
    source: r("tests/evidence/depth/probe-depth/smoke-tick120.png"),
    status: "interim",
    crop: LOOP_CROP,
    alt: { en: "The fixed game at tick 120: the crate has landed", zh: "修好后第 120 帧：箱子已落地" },
  },
  "loop-film": {
    source: p("studio-win: an agent building a game, split-screen timelapse"),
    status: "pending",
    alt: { en: "An agent building a game, sped up", zh: "AI 制作一款游戏的延时录像" },
  },

  // ── Rendering ─────────────────────────────────────────────────────────
  "render-refl-off": {
    source: s("depth/417337a4426c"),
    status: "interim",
    crop: STATION_CROP,
    alt: { en: "A chrome and a gold sphere with the reflection probe off", zh: "关闭反射探针的镜面球与金球" },
  },
  "render-refl-on": {
    source: s("depth/58cda7fcfcf5"),
    status: "interim",
    crop: STATION_CROP,
    alt: { en: "The same spheres reflecting the room through the probe", zh: "通过反射探针映出房间的同一组球" },
  },
  "render-ssr-off": {
    source: s("depth/d53d72bb373f"),
    status: "interim",
    crop: STATION_CROP,
    alt: { en: "The spheres with screen-space reflections off", zh: "关闭屏幕空间反射的球体" },
  },
  "render-ssr-on": {
    source: s("depth/034f077b7e97"),
    status: "interim",
    crop: STATION_CROP,
    alt: { en: "The spheres with screen-space reflections on", zh: "开启屏幕空间反射的球体" },
  },
  "render-fog-before": {
    source: s("debt/b42ff3bd7446"),
    status: "interim",
    alt: { en: "Light shafts before the volumetric fog rework", zh: "体积雾重做前的光柱" },
  },
  "render-fog-after": {
    source: s("debt/30e26752a704"),
    status: "interim",
    alt: { en: "Light shafts after the volumetric fog rework", zh: "体积雾重做后的光柱" },
  },
  // One avenue, one camera, seven atmospheres (og-tests reference frames).
  "weather-fair": { source: ref("clouds-fair"), status: "interim", alt: { en: "Fair-weather cumulus", zh: "晴天积云" } },
  "weather-sunset": { source: ref("clouds-sunset"), status: "interim", alt: { en: "Clouds at sunset", zh: "日落时的云" } },
  "weather-overcast": { source: ref("clouds-overcast"), status: "interim", alt: { en: "Overcast", zh: "阴天" } },
  "weather-rain": { source: ref("weather-rain"), status: "interim", alt: { en: "Rain", zh: "雨" } },
  "weather-lightning": { source: ref("weather-lightning"), status: "interim", alt: { en: "A lightning strike at night", zh: "夜晚的闪电" } },
  "weather-snow": { source: ref("weather-snow-cover"), status: "interim", alt: { en: "Snow cover", zh: "积雪" } },
  "weather-dust": { source: ref("atmosphere-dusk"), status: "interim", alt: { en: "Dust haze at dusk", zh: "黄昏沙尘" } },
  "water-sunset": { source: ref("water-ocean-sunset"), status: "interim", alt: { en: "Open ocean at sunset", zh: "日落时的海面" } },
  "water-storm": { source: ref("water-ocean-storm"), status: "interim", alt: { en: "Open ocean in a gale", zh: "大风中的海面" } },
  "render-reel": {
    source: p("studio-win: rendering reel, 4K"),
    status: "pending",
    alt: { en: "Rendering reel", zh: "渲染集锦" },
  },

  // ── Characters ────────────────────────────────────────────────────────
  "char-autorig": {
    source: s("G3-1/d15d41a5d72c"),
    status: "interim",
    posterAt: 1.5,
    alt: { en: "A knight auto-rigged from one layered drawing, playing its ten clips", zh: "从一张分层原画自动绑骨的骑士，播放十段动作" },
  },

  "char-motion-course": {
    source: s("og.motion/8f9c1f54284d"),
    status: "interim",
    alt: { en: "Two agents running og.motion on the motion course", zh: "动作训练场上由 og.motion 驱动的两个角色" },
  },
  "char-cloth": {
    source: s("fx-showcase/282040ae7015"),
    status: "interim",
    crop: STATION_CROP,
    alt: { en: "A cloth banner draped over a pole", zh: "搭在杆上的布料" },
  },
  "motion-poster": {
    source: s("og.motion/8f9c1f54284d"),
    status: "interim",
    alt: { en: "Agents driven by og.motion on the motion course", zh: "动作训练场上由 og.motion 驱动的角色" },
  },
  "motion-film": {
    source: p("site-motion: a visitor steering the og.motion character with the step log, 15–20 s, 4K"),
    status: "pending",
    alt: { en: "Steering a character driven by og.motion", zh: "操控由 og.motion 驱动的角色" },
  },


  // ── Worlds ────────────────────────────────────────────────────────────
  "world-terrain": {
    source: s("G2-6/45b7d36ae102"),
    status: "interim",
    posterAt: 1,
    alt: { en: "Noise terrain regenerating as its seed drifts", zh: "随种子漂移实时重生的噪声地形" },
  },
  "world-dungeon": {
    source: s("G2-6/bf839d77fec4"),
    status: "interim",
    posterAt: 1,
    alt: { en: "BSP dungeons regenerated live", zh: "实时重生的 BSP 地牢" },
  },
  "world-village": {
    source: s("G2-6/5729b0364c18"),
    status: "interim",
    posterAt: 1,
    alt: { en: "Wave-function-collapse villages regenerated live", zh: "实时重生的 WFC 村庄" },
  },
  "world-far": { source: ref("terrain-far"), status: "interim", alt: { en: "An island terrain seen from far above", zh: "高空俯瞰的岛屿地形" } },
  "world-grass": { source: ref("terrain-grass-wind"), status: "interim", alt: { en: "Wind through terrain grass under mountains", zh: "山下随风摆动的草地" } },
} satisfies Record<string, Slot>;

export type SlotName = keyof typeof media;
