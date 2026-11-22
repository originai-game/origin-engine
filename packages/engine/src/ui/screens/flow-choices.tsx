import { For, Show, Text, View, createMemo, t, type UiEvent } from "@origingame/engine/ui";
import type { FlowView } from "@origingame/engine";
import { Progress } from "../controls/progress";
import "./flow-choices.en.ogt";

export interface FlowChoicesProps {
  /** What the flow shows now (og.flow.view's value, set each tick); null or a view without a window hides the screen. */
  view: FlowView | null;
  /** A choice was taken, by its row or its hotspot: pass `index` (ChoiceView.index) to og.flow.choose. */
  onChoose?: ((e: UiEvent, index: number) => void) | undefined;
}

type Row = { index: number; text: string; enabled: boolean; chosen: boolean; progress: number };
type Spot = { index: number; x: number; y: number; width: number; height: number; label: string };

/** The rows a view shows: its choices with the pending one marked and each QTE's progress (-1: none), and its hotspots. */
function rows(view: FlowView | null): { open: boolean; left: number; locked: boolean; choices: Row[]; spots: Spot[] } {
  const win = view?.window ?? null;
  if (!view || !win) return { open: false, left: 0, locked: false, choices: [], spots: [] };
  return {
    open: true,
    left: win.ticksTotal > 0 ? win.ticksLeft / win.ticksTotal : 0,
    locked: win.locked,
    choices: view.choices.map((c) => ({
      index: c.index,
      text: c.text,
      enabled: c.enabled,
      chosen: c.index === win.pending,
      progress: view.qtes.find((q) => q.choice === c.index)?.progress ?? -1,
    })),
    spots: view.hotspots.map((h) => ({
      index: h.choice,
      x: h.x,
      y: h.y,
      width: h.width,
      height: h.height,
      label: view.choices.find((c) => c.index === h.choice)?.text ?? "",
    })),
  };
}

/**
 * An interactive film's choice window over its footage (flow guide §4): a
 * thin bar over the choices drains with the time left; the choices are cut
 * plates docked at the foot, the pending one ringed in the accent, one that
 * cannot be taken dim, a quick-time one with its progress under its words.
 * Hotspots are lit frames at the fractions of the picture the flow gives,
 * so they land on it when the game draws the picture to fill the surface.
 * Past the lock the choices stay and take nothing.
 */
export function FlowChoices(props: FlowChoicesProps) {
  const now = createMemo(() => rows(props.view));
  return (
    <Show when={now().open}>
      <View class="absolute inset-0">
        <View class="absolute inset-0">
          <For each={now().spots} key={(s) => s.index}>
            {(s) => (
              <View
                class="flex-row items-center justify-center gap-1 absolute min-w-11 min-h-11 rounded-control ring-2 ring-gold bg-material-thin hover:bg-material-regular disabled:opacity-45 disabled:ring-rim"
                role="button"
                label={s().label}
                disabled={now().locked}
                style={{ left: `${s().x * 100}%`, top: `${s().y * 100}%`, width: `${s().width * 100}%`, height: `${s().height * 100}%` }}
                onPress={(e) => { if (!now().locked) props.onChoose?.(e, s().index); }}
              />
            )}
          </For>
        </View>
        <View class="absolute inset-x-0 bottom-0 items-center bg-linear-to-t from-shade-deep to-transparent pt-12 pb-[calc(env(hud-inset-bottom)+24px)] pl-[calc(env(hud-inset-left)+48px)] pr-[calc(env(hud-inset-right)+48px)] compact:px-4 compact:pb-[calc(env(hud-inset-bottom)+16px)]">
          <View class="w-full max-w-240 gap-2" role="group" label={t("screen.flow-choices.label")}>
            <Progress value={now().left} class="h-1 rounded-none" />
            <View class="flex-row flex-wrap gap-2 compact:flex-col compact:flex-nowrap">
              <For each={now().choices} key={(c) => c.index}>
                {(c) => (
                  <View
                    class="grow shrink basis-42 min-h-14 items-center justify-center gap-1 py-2 px-3 rounded-tl-block rounded-br-block bg-plate-lo shadow-plate text-fg hover:bg-plate-hi data-chosen:ring-2 data-chosen:ring-accent data-chosen:bg-accent data-chosen:text-accent-fg disabled:opacity-45 compact:basis-auto"
                    role="button"
                    current={c().chosen}
                    data-chosen={c().chosen}
                    disabled={!c().enabled || now().locked}
                    onPress={(e) => { if (c().enabled && !now().locked) props.onChoose?.(e, c().index); }}
                  >
                    <Text class="text-lg font-semibold leading-snug text-center">{c().text}</Text>
                    <Show when={c().progress >= 0}>
                      <Progress value={c().progress} label={t("screen.flow-choices.qte")} class="w-full h-1.5 rounded-none" />
                    </Show>
                  </View>
                )}
              </For>
            </View>
          </View>
        </View>
      </View>
    </Show>
  );
}

const view = (pending: number | null, locked = false) => ({
  flow: 1, node: "Kitchen", segment: "segments/kitchen.ogt", texture: 0, time: 9, cut: 14,
  choices: [
    { index: 0, text: "Take the knife", enabled: true },
    { index: 1, text: "Open the door", enabled: true },
    { index: 2, text: "Hold to dodge", enabled: true },
    { index: 3, text: "Wait in the dark", enabled: false },
  ],
  window: { ticksLeft: 180, ticksTotal: 300, defaultChoice: 1, pending, locked },
  hotspots: [{ choice: 0, id: "knife", x: 0.58, y: 0.3, width: 0.14, height: 0.22 }],
  qtes: [{ choice: 2, kind: "hold", action: "confirm", progress: 0.45, done: false }],
  previous: 0, mix: 1, paused: false, chapter: "", ending: "", ended: false,
}) as unknown as FlowView;

export const fixtures: Record<string, Partial<FlowChoicesProps>> = {
  default: { view: view(1) },
  locked: { view: view(1, true) },
  closed: { view: null },
};
