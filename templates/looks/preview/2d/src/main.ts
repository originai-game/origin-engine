// The 2D looks' preview scene (templates/looks/preview/README.md): one
// place drawn in every 2D look, as a game's screen. From the side: a hooded
// traveller with a lantern on a level's ground (a tilemap) under a floating
// platform with gems over it, a street lantern, a crate at the foot of a
// ledge with a slime on it, trees, a tree line and the far valley behind,
// the lanterns' 2D light and a HUD. The `top` plan is the same place from
// above: the meadow and its paved path, the plateau's cliff, the same
// traveller, lantern, crate, slime, gems and trees.
//
// The art is assets/art/ (the look's own, copied in by stage.ts), each beat
// at the size src/art.ts gives it; the cook conforms each texture to the
// look's art contract and the look's frame does the rest.
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { ART, HAZE, LIGHT, PLAN, TEXT, UNIT, VIEW_HEIGHT } from "./art.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const VIEW = { w: (VIEW_HEIGHT * 16) / 9, h: VIEW_HEIGHT };
/** The view's centre: the side level's ground line (y 0) lies three tenths up from its bottom; from above, the path lies a little under the middle. */
const CENTRE = { x: 0, y: PLAN === "side" ? VIEW.h * 0.3 : 1 };
/** Sprite layers: what 2D light reaches lies from LIT up to the HUD. */
const LIT = -15;
const HUD = 200;
/** At night the lanterns light the scene; by day they only warm what they reach. */
const DARK = LIGHT === "night";

/** A length on the look's pixel grid (as it is without one). */
const snap = (v: number) => (UNIT ? Math.round(v * UNIT) / UNIT : v);

interface Extra {
  layer?: number;
  flipX?: boolean;
  sortY?: boolean;
  shadow?: number;
  glow?: number;
  opacity?: number;
  /** The quad's point at (x, y), from its bottom-left corner. */
  anchor?: { x: number; y: number };
  width?: number;
  height?: number;
}

/** A beat's sprite standing on (x, y), its corner on the pixel grid. */
function sprite(world: World, beat: string, x: number, y: number, extra: Extra = {}) {
  const art = ART[beat];
  if (!art) return null;
  const width = extra.width ?? art.width;
  const height = extra.height ?? art.height;
  const anchor = extra.anchor ?? { x: 0.5, y: 0 };
  const texture = must(og.load.texture(world, art.file));
  const position = { x: snap(x - width * anchor.x) + width * anchor.x, y: snap(y - height * anchor.y) + height * anchor.y, z: 0 };
  const e = must(world.spawn({ name: `${beat}@${x},${y}`, position }));
  must(world.insert(e, og.Sprite, { texture, width, height, anchor, layer: extra.layer ?? 0, flipX: extra.flipX ?? false, sortY: extra.sortY ?? false, shadow: extra.shadow ?? 0, glow: extra.glow ?? 0, opacity: extra.opacity ?? 1 }));
  return e;
}

/** A warm point light: a lantern's. */
function lamp(world: World, x: number, y: number, intensity: number, distance: number) {
  if (LIGHT === "none") return;
  const e = must(world.spawn({ name: `light@${x},${y}`, position: { x, y, z: 0 } }));
  must(world.insert(e, og.Light2D, { color: { r: 1, g: 0.62, b: 0.28 }, intensity, distance, falloff: 1.5, sourceRadius: 0.3, castShadows: true }));
}

/** Something that stops 2D light: the ledge, the platform, a crate. */
function occluder(world: World, x: number, y: number, width: number, height: number) {
  const e = must(world.spawn({ name: `occluder@${x},${y}`, position: { x, y, z: 0 } }));
  must(world.insert(e, og.Occluder2D, { width, height }));
}

function light(world: World) {
  if (LIGHT === "none") return;
  const l = must(world.resource(og.Lighting2D));
  l.enabled = true;
  l.minLayer = LIT;
  l.maxLayer = HUD - 1;
  l.resolution = 1;
  // By day the ambient light is white, so the art keeps the colours the cook
  // conformed and only what a lantern reaches is warmed.
  l.ambient = DARK ? { r: 0.6, g: 0.66, b: 0.9 } : { r: 1, g: 1, b: 1 };
}

/** The level's map with its top-left corner at (x, y). */
function level(world: World, x: number, y: number) {
  const map = must(og.load.tilemap(world, "base/level/level"));
  const e = must(world.spawn({ name: "level", position: { x, y, z: 0 } }));
  must(world.insert(e, og.Tilemap, { map: map.map, layer: -10, collision: false }));
}

/** The far layer filling the view, unlit. */
function backdrop(world: World) {
  const back = ART.backdrop;
  if (!back) return;
  const fill = Math.max(VIEW.w / back.width, VIEW.h / back.height);
  sprite(world, "backdrop", CENTRE.x, CENTRE.y, { layer: -30, anchor: { x: 0.5, y: 0.5 }, width: back.width * fill, height: back.height * fill });
}

/** The HUD's own measures: a margin from the view's corner, and the gem count's size. */
const HUD_AT = { left: CENTRE.x - VIEW.w / 2, right: CENTRE.x + VIEW.w / 2, top: CENTRE.y + VIEW.h / 2, margin: VIEW.h * 0.04, count: (ART.gem?.height ?? 1) * 1.1 };

/** The HUD: three hearts top left, the gems collected top right (the count is the Hud system's). */
function hud(world: World) {
  const { left, right, top, margin, count } = HUD_AT;
  const heart = ART.heart;
  if (heart) for (let i = 0; i < 3; i++) sprite(world, "heart", left + margin + heart.width * (0.5 + i * 1.2), top - margin - heart.height, { layer: HUD });
  const gem = ART.gem;
  if (gem) sprite(world, "gem", right - margin - count * 1.9 - gem.width / 2, top - margin - gem.height, { layer: HUD });
}

function side(world: World) {
  backdrop(world);
  // The tree line, between the far layer and the level: its copies either side are mirrored, so they join.
  const mid = ART.mid;
  if (mid) for (const i of [-1, 0, 1]) sprite(world, "mid", 1 + i * mid.width, -0.5, { layer: -20, flipX: i !== 0, opacity: HAZE });
  level(world, -20, 2);

  for (const x of [-7.5, 9.5, -12]) sprite(world, "tree", x, x > 4 ? 2 : 0, { layer: -4, flipX: x > 0 });
  sprite(world, "lantern", 0.75, 0, { layer: -3 });
  sprite(world, "lantern", 11.5, 2, { layer: -3, flipX: true });
  sprite(world, "platform", -3, 3, { layer: -2 });
  if (ART.platform) occluder(world, -3, 3 + ART.platform.height / 2, ART.platform.width, ART.platform.height);
  sprite(world, "crate", 3.25, 0, { layer: 1 });
  if (ART.crate) occluder(world, 3.25, ART.crate.height / 2, ART.crate.width, ART.crate.height);
  // At night the slime glows, as the gems do.
  sprite(world, "slime", 6.25, 2, { layer: 2, glow: DARK ? 0.4 : 0 });
  const over = 3 + (ART.platform?.height ?? 1) + 0.5;
  for (const [x, y] of [[-4, over], [-3, over], [-2, over], [3.25, 2.25], [8, 3.25]] as const) sprite(world, "gem", x, y, { layer: 3, glow: DARK ? 1.6 : 0 });
  sprite(world, "hero", -2.25, 0, { layer: 5 });
  for (const [x, y, flip] of [[-5.25, 0, false], [1.9, 0, true], [4.9, 2, false], [-10, 0, true]] as const) sprite(world, "bush", x, y, { layer: 8, flipX: flip });

  light(world);
  const post = ART.lantern;
  if (post) {
    lamp(world, 0.75, post.height * 0.8, DARK ? 1.3 : 0.6, DARK ? 8 : 6.5);
    lamp(world, 11.5, 2 + post.height * 0.8, DARK ? 1.3 : 0.6, DARK ? 8 : 6.5);
  }
  if (ART.hero) lamp(world, -2.25 + ART.hero.width * 0.38, ART.hero.height * 0.42, DARK ? 0.9 : 0.45, DARK ? 4.5 : 3.5);
}

function top(world: World) {
  level(world, -14, 8);
  // The plateau's west edge: a rim of the cliff's rock, since no face shows from above.
  light(world);
  const stand = { layer: 0, sortY: true };
  for (const [x, y] of [[-7, 2.75], [-11, -3.25], [8.5, -3.5], [11.25, 4.5], [-3.25, 4.75]] as const) sprite(world, "tree", x, y, { ...stand, shadow: 0.35 });
  sprite(world, "lantern", 0.5, 1, { ...stand, shadow: 0.3 });
  sprite(world, "crate", 5.5, 0.9, { ...stand, shadow: 0.3 });
  // The plateau's slime stands clear of the HUD's corner, and no gem lies under it.
  sprite(world, "slime", 6.5, 4.3, { ...stand, shadow: 0.3 });
  for (const [x, y] of [[-6.5, -0.75], [-5.25, -0.75], [-4, -0.75], [2.5, -2.75], [8, 0.75]] as const) sprite(world, "gem", x, y, { ...stand, glow: DARK ? 0.6 : 0 });
  sprite(world, "hero", -2, -0.9, { ...stand, shadow: 0.35 });
  for (const [x, y, flip] of [[-9.5, 3.5, false], [-4.75, 1.4, true], [5, -2.6, false], [-1.5, -4.25, true], [10.5, 0.6, false], [-12, 0.9, true]] as const) sprite(world, "bush", x, y, { ...stand, flipX: flip });
  const post = ART.lantern;
  // From above a lantern's pool lies on the ground round its foot.
  if (post) lamp(world, 0.5, 1.25, DARK ? 1.3 : 0.6, DARK ? 8 : 5);
  if (ART.hero) lamp(world, -2 + ART.hero.width * 0.3, -0.9 + ART.hero.height * 0.4, DARK ? 0.9 : 0.4, DARK ? 4.5 : 3);
}

/**
 * The HUD's count, drawn each tick in screen space at the frame's own
 * resolution, as a game draws its HUD's text: over the finished frame, so
 * the look's grid and dither never reach the glyphs. It stands where the
 * camera puts the HUD's world points, beside the gem.
 */
const Hud = defineSystem("Hud", { reads: [og.Camera] }, (_q, _time, world) => {
  const { right, top, margin, count } = HUD_AT;
  const cam = must(world.resource(og.Camera));
  const baseline = { x: snap(right - margin - count * 1.7), y: snap(top - margin - (ART.gem?.height ?? 1) * 0.88), z: 0 };
  const at = cam.worldToScreen(baseline);
  const up = cam.worldToScreen({ x: baseline.x, y: baseline.y + 1, z: 0 });
  if (!at || !up) return;
  // CSS pixels a metre.
  const scale = at.y - up.y;
  const font = must(og.load.font(world, "base/fonts/NotoSans-Latin"));
  const size = Math.round(count * scale);
  const rim = Math.max(1, Math.round(size / 14));
  const text = { font, text: "× 07", size, screen: true };
  // The rim: the count in the rim's colour, a step each way round it.
  for (const [dx, dy] of [[-1, -1], [0, -1], [1, -1], [-1, 0], [1, 0], [-1, 1], [0, 1], [1, 1]] as const) {
    must(og.render.text(world, { ...text, position: { x: Math.round(at.x) + dx * rim, y: Math.round(at.y) + dy * rim, z: 0 }, color: TEXT.rim, layer: HUD }));
  }
  must(og.render.text(world, { ...text, position: { x: Math.round(at.x), y: Math.round(at.y), z: 0 }, color: TEXT.color, layer: HUD + 1 }));
});

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/main.ogt",
  systems: [Hud],
  start(world) {
    const cam = must(world.resource(og.Camera));
    cam.orthographic = true;
    cam.height = VIEW.h;
    cam.position = { x: CENTRE.x, y: snap(CENTRE.y), z: 50 };
    cam.background = { r: 0.02, g: 0.02, b: 0.05 };
    if (PLAN === "top") top(world);
    else side(world);
    if (DARK) must(world.resource(og.PostEffects)).glow = true;
    hud(world);
  },
});
