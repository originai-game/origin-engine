import type * as Api from "@origingame/engine";
import { type Custom, TweenTimeline } from "../generated/calls.ts";
import { registerCommand } from "./declare.ts";
import { abi } from "./host.ts";
import { json, parse } from "./result.ts";
import { checkAccess } from "./views.ts";
import { uiReactor } from "./world.ts";
import { math } from "../math.ts";

type W = { readonly id: number };

// ── the `custom` members of the plain modules (IDL `audio`, `commands`, `tween`) ──

export const audioCustom: Custom["audio"] = {
  outputLatency: (world) => (checkAccess("read", "og.Audio"), abi().og_audio_output_latency(world.id)),
};

export const commandsCustom: Custom["commands"] = {
  register: (name, handler, desc) => registerCommand(name, handler, desc as never),
};

export const tweenTimelineCustom: Custom["TweenTimeline"] = {
  // A timeline object nests by its handle, as GSAP's tl.add(child) does.
  add: (tl, child, position) => {
    const c = typeof child === "object" && child !== null && "tween" in child ? (child as { tween: number }).tween : child;
    abi().og_tween_timeline_add(TweenTimeline.handleOf(tl), JSON.stringify(c), json(position));
    return tl;
  },
};

/** One lookup for a flipbook's or a 2D rig's clips (an index, -1 when missing) and a loaded model's (og.load.animation: a ClipAsset, 0 when missing): `og.anim.clip` (IDL `host false`). */
export const clip = (info: { clips?: ReadonlyArray<{ clip: number; name: string }>; skeleton?: number }, name: string): number =>
  info?.clips?.find((c) => c.name === name)?.clip ?? (info?.skeleton !== undefined ? 0 : -1);

/**
 * The Environment.rotation that puts a cooked map's sun at `azimuth` around `view` (`og.load.environmentSunRotation`,
 * IDL `host false`): the sun's horizontal bearing at rotation 0 (og.load.environmentSun) turned onto the bearing
 * `azimuth` from the view's, positive to its right. A turn about +Y keeps the sun's elevation, which is reported.
 * Computed with `math`, so a system that sets the rotation from it gives the same world on every host.
 */
export const environmentSunRotation = (
  world: W,
  map: number,
  view: { x: number; y: number; z: number },
  azimuth: number,
): Api.Result<Api.EnvironmentSunTurn> => {
  const answered = parse<Api.Result<Api.EnvironmentSun>>(abi().og_load_environment_sun(world.id, map, 0, 1));
  if (!answered.ok) return answered;
  const sun = answered.value;
  if (!(sun.intensity > 0)) {
    return { ok: false, reason: "the map holds no sun: its cook found none (og cook's environment.sun.missing says why) or it was cooked with `sun keep`" };
  }
  const flat = Math.sqrt(sun.direction.x * sun.direction.x + sun.direction.z * sun.direction.z);
  if (!(flat > 1e-4)) return { ok: false, reason: "the sun stands straight overhead: no turn about +Y moves it" };
  const viewFlat = Math.sqrt(view.x * view.x + view.z * view.z);
  if (!(viewFlat > 1e-6)) return { ok: false, reason: "the view looks straight up or down: it has no horizontal direction to turn the sun around" };
  if (!Number.isFinite(azimuth)) return { ok: false, reason: `azimuth ${azimuth} is not a finite number of radians` };
  // A bearing as Environment.rotation counts it: the angle from +X towards -Z, which a rotation adds to.
  const bearing = (x: number, z: number) => math.atan2(-z, x);
  const fx = view.x / viewFlat, fz = view.z / viewFlat;
  // The view's right is forward × up: (-fz, 0, fx).
  const tx = fx * math.cos(azimuth) - fz * math.sin(azimuth);
  const tz = fz * math.cos(azimuth) + fx * math.sin(azimuth);
  let rotation = bearing(tx, tz) - bearing(sun.direction.x, sun.direction.z);
  if (rotation > Math.PI) rotation -= 2 * Math.PI;
  else if (rotation <= -Math.PI) rotation += 2 * Math.PI;
  const c = math.cos(rotation), s = math.sin(rotation);
  const direction = { x: sun.direction.x * c + sun.direction.z * s, y: sun.direction.y, z: sun.direction.z * c - sun.direction.x * s };
  return { ok: true, value: { rotation, direction, elevation: math.atan2(sun.direction.y, flat) } };
};

export const ui = Object.freeze({
  scope: (world: W, id: string): Api.UiScope => uiReactor(world).scope(id),
  configure: (world: W, desc?: unknown) => parse(abi().og_ui_configure(world.id, json(desc))),
  viewport: (world: W) => parse(abi().og_ui_viewport(world.id)),
  theme: (world: W) => parse(abi().og_ui_theme(world.id)),
  frameDirection: (): "ltr" | "rtl" => abi().og_ui_frame_direction() as "ltr" | "rtl",
  frameCursor: (): Api.Cursor => abi().og_ui_frame_cursor() as Api.Cursor,
  feedback: (event: string, rise: number) => abi().og_ui_feedback(event, rise),
  setHaptics: (intensity: number) => abi().og_ui_set_haptics(intensity),
});
