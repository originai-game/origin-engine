// The game widget set's host calls (IDL module `ui`, crates/og-ui) and the
// TypeScript sugar over them: anchor, transition, modal, list and grid take
// a build callback and close what they open even when it throws. The screen
// templates (./templates) are written against `kit`, which is og.ui with
// these members added.

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
  breakpoint: (): Api.Breakpoint => abi().og_ui_breakpoint() as Api.Breakpoint,
  frameViewport: (): Api.Viewport => parse(abi().og_ui_frame_viewport()),
  frameTheme: (): Api.UiTheme => parse(abi().og_ui_frame_theme()),
  hasIcon: (name: string): boolean => abi().og_ui_has_icon(name),
  seconds: (): number => abi().og_ui_seconds(),
  openAnchor: (id: string, desc?: Api.AnchorDesc): void => abi().og_ui_open_anchor(id, json(desc)),
  anchor: (id: string, desc: Api.AnchorDesc, build: Api.UiBuild): void =>
    within(() => abi().og_ui_open_anchor(id, json(desc)), build, () => abi().og_ui_close()),
  openTransition: (id: string, show: boolean, desc?: Api.TransitionDesc): Api.Presence =>
    parse(abi().og_ui_open_transition(id, show, json(desc))),
  transition: (id: string, show: boolean, desc: Api.TransitionDesc, build: Api.UiBuild): Api.Presence => {
    const p = parse<Api.Presence>(abi().og_ui_open_transition(id, show, json(desc)));
    within(() => {}, () => { if (p.visible) build(); }, () => abi().og_ui_close());
    return p;
  },
  openModal: (id: string, open: boolean, desc?: Api.ModalDesc): Api.ModalResult => parse(abi().og_ui_open_modal(id, open, json(desc))),
  closeModal: (): void => abi().og_ui_close_modal(),
  modal: (id: string, open: boolean, desc: Api.ModalDesc, build: Api.UiBuild): Api.ModalResult => {
    const r = parse<Api.ModalResult>(abi().og_ui_open_modal(id, open, json(desc)));
    within(() => {}, () => { if (r.visible) build(); }, () => abi().og_ui_close_modal());
    return r;
  },
  spotlight: (id: string, hole: Api.UiRect | null, desc?: Api.SpotlightDesc): Api.Response =>
    parse(abi().og_ui_spotlight(id, JSON.stringify(hole ?? null), json(desc))),
  bounds: (path: string): Api.UiRect | null => parse(abi().og_ui_bounds(path)),
  openList: (id: string, desc?: Api.ListDesc): Api.ListRange => parse(abi().og_ui_open_list(id, json(desc))),
  openGrid: (id: string, desc?: Api.GridDesc): Api.ListRange => parse(abi().og_ui_open_grid(id, json(desc))),
  listItem: (index: number, selected: boolean): Api.Response => parse(abi().og_ui_list_item(index, selected)),
  gridItem: (index: number, selected: boolean): Api.Response => parse(abi().og_ui_grid_item(index, selected)),
  closeList: (): Api.Response => parse(abi().og_ui_close_list()),
  dragging: (id: string): number | null => parse(abi().og_ui_dragging(id)),
  list: (id: string, desc: Api.ListDesc, build: Api.UiItemBuild): Api.ListResult => items(id, desc, build, false),
  grid: (id: string, desc: Api.GridDesc, build: Api.UiItemBuild): Api.ListResult => items(id, desc, build, true),
  tabs: (id: string, items: readonly string[], selected: number, desc?: Api.TabsDesc): Api.ChoiceResult =>
    parse(abi().og_ui_tabs(id, JSON.stringify(items), selected, json(desc))),
  select: (id: string, options: readonly string[], selected: number, desc?: Api.SelectDesc): Api.ChoiceResult =>
    parse(abi().og_ui_select(id, JSON.stringify(options), selected, json(desc))),
  stepper: (id: string, value: number, desc?: Api.StepperDesc): Api.StepperResult => parse(abi().og_ui_stepper(id, value, json(desc))),
  radio: (id: string, options: readonly string[], selected: number, desc?: Api.RadioDesc): Api.ChoiceResult =>
    parse(abi().og_ui_radio(id, JSON.stringify(options), selected, json(desc))),
  checkbox: (id: string, on: boolean, label: string): Api.ToggleResult => parse(abi().og_ui_checkbox(id, on, label)),
  checkboxList: (id: string, labels: readonly string[], values: readonly boolean[]): Api.CheckListResult =>
    parse(abi().og_ui_checkbox_list(id, JSON.stringify(labels), JSON.stringify(values))),
  chip: (id: string, text: string, desc?: Api.ChipDesc): Api.Response => parse(abi().og_ui_chip(id, text, json(desc))),
  badge: (id: string, text: string, tone: Api.Tone): void => abi().og_ui_badge(id, text, tone),
  divider: (id: string, vertical: boolean): void => abi().og_ui_divider(id, vertical),
  spacer: (id: string, size: Api.Size | null): void => abi().og_ui_spacer(id, JSON.stringify(size ?? null)),
  keyValue: (id: string, key: string, value: string, mono: boolean): void => abi().og_ui_key_value(id, key, value, mono),
  skeleton: (id: string, desc?: Api.SkeletonDesc): void => abi().og_ui_skeleton(id, json(desc)),
  spinner: (id: string, size: number): void => abi().og_ui_spinner(id, size),
  icon: (id: string, name: string, desc?: Api.IconDesc): void => abi().og_ui_icon(id, name, json(desc)),
  meter: (id: string, value: number, max: number, desc?: Api.MeterDesc): void => abi().og_ui_meter(id, value, max, json(desc)),
  cooldown: (id: string, remaining: number, desc?: Api.CooldownDesc): Api.Response => parse(abi().og_ui_cooldown(id, remaining, json(desc))),
  minimap: (id: string, desc?: Api.MinimapDesc): void => abi().og_ui_minimap(id, json(desc)),
  zoomView: (id: string, desc?: Api.ZoomViewDesc): Api.ZoomResult => parse(abi().og_ui_zoom_view(id, json(desc))),
  dialogue: (id: string, desc?: Api.DialogueDesc): Api.DialogueResult => parse(abi().og_ui_dialogue(id, json(desc))),
  joystick: (id: string, desc?: Api.JoystickDesc): Api.JoystickResult => parse(abi().og_ui_joystick(id, json(desc))),
  dpad: (id: string, desc?: Api.DpadDesc): Api.DpadResult => parse(abi().og_ui_dpad(id, json(desc))),
  actionButton: (id: string, text: string, desc?: Api.ActionButtonDesc): Api.Response => parse(abi().og_ui_action_button(id, text, json(desc))),
  toast: (world: Api.World, text: string, desc?: Api.ToastDesc): number => abi().og_ui_toast(worldId(world), text, json(desc)),
  dismissToast: (world: Api.World, toastId: number): void => abi().og_ui_dismiss_toast(worldId(world), toastId),
  navigate: (world: Api.World, intent: Api.NavIntent): void => abi().og_ui_navigate(worldId(world), intent),
  back: (): boolean => abi().og_ui_back(),
  setNavigation: (on: boolean): void => abi().og_ui_set_navigation(on),
  setPlayers: (count: number): void => abi().og_ui_set_players(count),
  focus: (pad: number): string => abi().og_ui_focus(pad),
  a11y: (): Api.A11yTree => parse(abi().og_ui_a11y()),
  memory: (id: string, initial: number): number => abi().og_ui_memory(id, initial),
  remember: (id: string, value: number): void => abi().og_ui_remember(id, value),
  tick: (): number => abi().og_ui_tick(),
  counter: (id: string, value: number, desc?: Api.CounterDesc): number => abi().og_ui_counter(id, value, json(desc)),
  burst: (desc?: Api.BurstDesc): void => abi().og_ui_burst(json(desc)),
  shake: (desc?: Api.ShakeDesc): void => abi().og_ui_shake(json(desc)),
  screenTransition: (id: string, screen: string, desc?: Api.ScreenTransitionDesc): Api.ScreenTransitionResult =>
    parse(abi().og_ui_screen_transition(id, screen, json(desc))),
  previewViewport: (world: Api.World, viewport: Api.Viewport | null): void =>
    abi().og_ui_preview_viewport(worldId(world), JSON.stringify(viewport ?? null)),
};

/** og.ui.list / og.ui.grid: open, build the items in view, collect clicks and drops, close. */
function items(id: string, desc: Api.ListDesc | Api.GridDesc, build: Api.UiItemBuild, grid: boolean): Api.ListResult {
  const range = grid ? widgets.openGrid(id, desc as Api.GridDesc) : widgets.openList(id, desc as Api.ListDesc);
  const selected = desc.selected ?? null;
  let clicked: number | null = null;
  let dropFrom: number | null = null;
  let dropTo: number | null = null;
  try {
    for (let i = range.first; i < range.last; i++) {
      const r = grid ? widgets.gridItem(i, i === selected) : widgets.listItem(i, i === selected);
      if (r.clicked) clicked = i;
      if (r.dropped !== null && r.dropped !== undefined) {
        dropFrom = r.dropped;
        dropTo = i;
      }
      try {
        build(i);
      } finally {
        base.close();
      }
    }
  } finally {
    widgets.closeList();
  }
  return { clicked, dropFrom, dropTo };
}

/** og.ui as the templates use it: the core calls and the widget set. */
export const kit = { ...base, ...widgets };
export type Kit = typeof kit;
