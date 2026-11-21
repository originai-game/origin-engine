import type * as Api from "@origingame/engine";
import { type Custom, TweenTimeline } from "../generated/calls.ts";
import { registerCommand } from "./declare.ts";
import { abi } from "./host.ts";
import { fromBase64, json, parse } from "./result.ts";
import { checkAccess } from "./views.ts";
import { uiReactor } from "./world.ts";

type W = { readonly id: number };

// ── the `custom` members of the plain modules (IDL `load`, `audio`, `commands`, `tween`) ──

export const loadCustom: Custom["load"] = {
  // Bytes cross as base64, as a room message's do.
  dataBytes: (world, asset) => {
    const r = parse<{ ok: true; value: string } | { ok: false; reason: string }>(abi().og_load_data_bytes(world.id, asset));
    return r.ok ? { ok: true, value: fromBase64(r.value) } : r;
  },
};

export const audioCustom: Custom["audio"] = {
  outputLatency: (world) => (checkAccess("read", "og.Audio"), abi().og_audio_output_latency(world.id)),
  // A typed array of samples crosses as a plain array.
  createSound: (world, desc) => {
    const d = desc !== null && typeof desc === "object" ? { ...(desc as Record<string, unknown>) } : desc;
    if (d !== null && typeof d === "object" && ArrayBuffer.isView((d as Record<string, unknown>).samples)) {
      const r = d as Record<string, unknown>;
      r.samples = Array.from(r.samples as ArrayLike<number>);
    }
    return parse(abi().og_audio_create_sound(world.id, json(d)));
  },
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
