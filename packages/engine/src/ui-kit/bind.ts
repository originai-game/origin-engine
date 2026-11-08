// The game widget set's host calls (IDL module `ui`, crates/og-ui) and the
// TypeScript sugar over them: anchor takes a build
// callback and close what they open even when it throws. The
// components (../ui) are written against `kit`, which is og.ui with these
// members added.

import type * as Api from "@origingame/engine";
import { boundHost, ui as base } from "../runtime.ts";

type HostAbi = NonNullable<ReturnType<typeof boundHost>>["abi"];

function abi(): HostAbi {
  const host = boundHost();
  if (!host) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return host.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));
const worldId = (world: Api.World): number => (world as unknown as { id: number }).id;

/**
 * The frame's theme and viewport, parsed only when their text changes: the
 * same object while the frame's style and surface stay the same (the
 * components read them many times a tick), and a fresh one the tick they
 * change, whichever engine is building.
 */
const lastParsed = new Map<string, { text: string; value: unknown }>();
function parsedOnce<T>(key: string, text: string): T {
  const last = lastParsed.get(key);
  if (last && last.text === text) return last.value as T;
  const value = parse<T>(text);
  lastParsed.set(key, { text, value });
  return value;
}

/** Runs `build` between an open and its close, closing even when it throws. */
function within(open: () => void, build: () => void, close: () => void): void {
  open();
  try {
    build();
  } finally {
    close();
  }
}

export const widgets = {
  frameViewport: (): Api.Viewport => parsedOnce("viewport", abi().og_ui_frame_viewport()),
  frameTheme: (): Api.UiTheme => parsedOnce("theme", abi().og_ui_frame_theme()),
  framePointer: (): Api.UiPointer => parse(abi().og_ui_frame_pointer()),
  setFocus: (path: string): void => abi().og_ui_set_focus(path),
  frameGlyph: (action: string): Api.UiGlyph | null => parse(abi().og_ui_frame_glyph(action)),
  hasIcon: (name: string): boolean => abi().og_ui_has_icon(name),
  seconds: (): number => abi().og_ui_seconds(),
  openAnchor: (id: string, desc?: Api.AnchorDesc): void => abi().og_ui_open_anchor(id, json(desc)),
  anchor: (id: string, desc: Api.AnchorDesc, build: Api.UiBuild): void =>
    within(() => abi().og_ui_open_anchor(id, json(desc)), build, () => abi().og_ui_close()),
  openModal: (id: string, open: boolean, desc?: Api.ModalDesc): Api.ModalResult => parse(abi().og_ui_open_modal(id, open, json(desc))),
  closeModal: (): void => abi().og_ui_close_modal(),
  bounds: (path: string): Api.UiRect | null => parse(abi().og_ui_bounds(path)),
  scrollState: (path: string): Api.ScrollState | null => parse(abi().og_ui_scroll_state(path)),
  divider: (id: string, vertical: boolean): void => abi().og_ui_divider(id, vertical),
  spacer: (id: string, size: Api.Size | null): void => abi().og_ui_spacer(id, JSON.stringify(size ?? null)),
  icon: (id: string, name: string, desc?: Api.IconDesc): void => abi().og_ui_icon(id, name, json(desc)),
  toast: (world: Api.World, text: string, desc?: Api.ToastDesc): number => abi().og_ui_toast(worldId(world), text, json(desc)),
  dismissToast: (world: Api.World, toastId: number): void => abi().og_ui_dismiss_toast(worldId(world), toastId),
  navigate: (world: Api.World, intent: Api.NavIntent): void => abi().og_ui_navigate(worldId(world), intent),
  back: (): boolean => abi().og_ui_back(),
  setNavigation: (on: boolean): void => abi().og_ui_set_navigation(on),
  setPlayers: (count: number): void => abi().og_ui_set_players(count),
  focus: (pad: number): string => abi().og_ui_focus(pad),
  // A pad's virtual cursor and the player's HUD layout (docs/ui.md §2.1, §2.1).
  setVirtualCursor: (desc: Api.VirtualCursorDesc | null): void =>
    abi().og_ui_set_virtual_cursor(JSON.stringify(desc ?? null)),
  virtualCursor: (): Api.VirtualCursor | null => parse(abi().og_ui_virtual_cursor()),
  setHudEdit: (on: boolean): void => abi().og_ui_set_hud_edit(on),
  setHudPlayer: (player: number): void => abi().og_ui_set_hud_player(player),
  hudEdit: (): Api.HudEdit => parse(abi().og_ui_hud_edit()),
  hudLayout: (player: number): string => abi().og_ui_hud_layout(player),
  setHudLayout: (player: number, layout: string): boolean => abi().og_ui_set_hud_layout(player, layout),
  hudSelect: (key: string): void => abi().og_ui_hud_select(key),
  hudStep: (scale: number, opacity: number): void => abi().og_ui_hud_step(scale, opacity),
  hudReset: (key: string): void => abi().og_ui_hud_reset(key),
  a11y: (): Api.A11yTree => parse(abi().og_ui_a11y()),
  memory: (id: string, initial: number): number => abi().og_ui_memory(id, initial),
  remember: (id: string, value: number): void => abi().og_ui_remember(id, value),
  tick: (): number => abi().og_ui_tick(),
  // World-space UI (docs/ui.md §1, §3.3).
  defineMass: (name: string, desc?: Api.MassTemplateDesc): Api.Result<number> => parse(abi().og_ui_define_mass(name, json(desc))),
  spawnMass: (template: number, instances: Api.MassInstance[]): void =>
    abi().og_ui_spawn_mass(template, JSON.stringify(instances)),
  massCount: (template: number): number => abi().og_ui_mass_count(template),
  previewViewport: (world: Api.World, viewport: Api.Viewport | null): void =>
    abi().og_ui_preview_viewport(worldId(world), JSON.stringify(viewport ?? null)),
  // The element model and motion (docs/ui.md §2, §3.1).
  props: (desc?: Api.ElementProps): boolean => abi().og_ui_props(json(desc)),
  defineTimeline: (name: string, timeline?: Api.UiTimeline): void => abi().og_ui_define_timeline(name, json(timeline)),
  play: (path: string, choreo: Api.Choreo, desc?: Api.PlayDesc): number => abi().og_ui_play(path, JSON.stringify(choreo), json(desc)),
  playTimeline: (path: string, timeline: string, desc?: Api.PlayDesc): number => abi().og_ui_play_timeline(path, timeline, json(desc)),
  seek: (play: number, ms: number): boolean => abi().og_ui_seek(play, ms),
  pause: (play: number): boolean => abi().og_ui_pause(play),
  resume: (play: number): boolean => abi().og_ui_resume(play),
  reverse: (play: number): boolean => abi().og_ui_reverse(play),
  stop: (play: number): boolean => abi().og_ui_stop(play),
  playInfo: (play: number): Api.PlayInfo | null => parse(abi().og_ui_play_info(play)),
  forceState: (path: string, group: string, state: string | null): void =>
    abi().og_ui_force_state(path, group, JSON.stringify(state ?? null)),
  visualState: (path: string, group: string): string => abi().og_ui_visual_state(path, group),
  openPath: (): string => abi().og_ui_open_path(),
  animate: (id: string, target: number, transition: Api.Transition): number => abi().og_ui_animate(id, target, JSON.stringify(transition)),
  gesture: (path: string): Api.GestureInfo => parse(abi().og_ui_gesture(path)),
  motionEvents: (): Api.UiMotionEvent[] => parse(abi().og_ui_motion_events()),
  overlaps: (): Api.UiOverlap[] => parse(abi().og_ui_overlaps()),
  layoutStats: (): Api.UiLayoutStats => parse(abi().og_ui_layout_stats()),
  // Documents, the layer stack, actions and glyphs (docs/ui.md §2.1, §2.1, §2.1).
  push: (world: Api.World, screen: string, vm?: unknown): Api.Result<number> =>
    parse(abi().og_ui_push(worldId(world), screen, json(vm))),
  pop: (world: Api.World): string => abi().og_ui_pop(worldId(world)),
  replace: (world: Api.World, screen: string, vm?: unknown): Api.Result<number> =>
    parse(abi().og_ui_replace(worldId(world), screen, json(vm))),
  clear: (world: Api.World): void => abi().og_ui_clear(worldId(world)),
  layers: (world: Api.World): readonly Api.UiLayer[] => parse(abi().og_ui_layers(worldId(world))),
  screenVm: (world: Api.World, screen: string): unknown => parse(abi().og_ui_screen_vm(worldId(world), screen)),
  docEvents: (world: Api.World): readonly Api.UiDocEvent[] => parse(abi().og_ui_doc_events(worldId(world))),
  gameInput: (world: Api.World): boolean => abi().og_ui_game_input(worldId(world)),
  paused: (world: Api.World): boolean => abi().og_ui_paused(worldId(world)),
  defineAction: (world: Api.World, desc: Api.UiActionDesc): Api.Result<void> =>
    parse(abi().og_ui_define_action(worldId(world), JSON.stringify(desc))),
  action: (name: string): boolean => abi().og_ui_action(name),
  actionHeld: (name: string): boolean => abi().og_ui_action_held(name),
  glyph: (world: Api.World, action: string): Api.UiGlyph | null => parse(abi().og_ui_glyph(worldId(world), action)),
  device: (world: Api.World): Api.UiDevice => abi().og_ui_device(worldId(world)) as Api.UiDevice,
  setDevice: (world: Api.World, device: Api.UiDevice): void => abi().og_ui_set_device(worldId(world), device),
  actionBar: (world: Api.World): readonly Api.UiActionHint[] => parse(abi().og_ui_action_bar(worldId(world))),
  documentSchema: (): unknown => parse(abi().og_ui_document_schema()),
  setProp: (world: Api.World, path: string, prop: string, value: string): Api.Result<void> =>
    parse(abi().og_ui_set_prop(worldId(world), path, prop, value)),
  saveEdits: (world: Api.World): Api.Result<readonly string[]> => parse(abi().og_ui_save_edits(worldId(world))),
};

/** og.ui as the templates use it: the core calls and the widget set. */
export const kit = { ...base, ...widgets };
export type Kit = typeof kit;
