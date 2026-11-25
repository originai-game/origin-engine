import { For, Show, Text, View, createMemo, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { CheckView, FlowView, QteView, TextSpan } from "@origingame/engine";
import { InputGlyph } from "../controls/input-glyph";
import { Progress } from "../controls/progress";
import "./flow-choices.en.ogt";

/** Rendered video rectangle in normalized surface fractions (`0..1`), as returned by `computeVideoBox`. */
export interface VideoBox {
  /** Left edge of the rendered video in surface width fractions. */
  left: number;
  /** Top edge of the rendered video in surface height fractions. */
  top: number;
  /** Width of the rendered video in surface width fractions. */
  width: number;
  /** Height of the rendered video in surface height fractions. */
  height: number;
}

export interface FlowChoicesProps {
  /** What the flow shows now (og.flow.view's value, set each tick); null or a view without a window/hotspots hides the screen. */
  view: FlowView | null;
  /** Rendered video rect in normalized surface fractions so hotspots align on any aspect ratio (16:9, 21:9, 9:16). */
  videoBox?: VideoBox | undefined;
  /** A choice was taken, by its row or its choice hotspot: pass `index` (ChoiceView.index) to og.flow.choose. */
  onChoose?: ((e: UiEvent, index: number) => void) | undefined;
  /** A non-cutting `inspect` hotspot was pressed: pass `hotspot` id to og.flow.inspect. */
  onInspect?: ((e: UiEvent, hotspot: string) => void) | undefined;
}

interface Row {
  index: number;
  text: string;
  spans: readonly TextSpan[];
  enabled: boolean;
  reason: string;
  visited: boolean;
  isDefault: boolean;
  check: CheckView | null;
  chosen: boolean;
  qte: QteView | null;
}

interface Spot {
  key: string;
  id: string;
  kind: "choice" | "inspect";
  shape: "rect" | "circle";
  index: number | null;
  left: number;
  top: number;
  width: number;
  height: number;
  label: string;
  inspected: boolean;
}

const DIR_ARROW: Record<string, string> = {
  up: "↑",
  down: "↓",
  left: "←",
  right: "→",
};

function qtePromptKey(kind: string): string {
  switch (kind) {
    case "press": return "screen.flow-choices.qte.press";
    case "hold": return "screen.flow-choices.qte.hold";
    case "mash": return "screen.flow-choices.qte.mash";
    case "timing": return "screen.flow-choices.qte.timing";
    case "sequence": return "screen.flow-choices.qte.sequence";
    case "swipe": return "screen.flow-choices.qte.swipe";
    case "drag": return "screen.flow-choices.qte.drag";
    default: return "screen.flow-choices.qte";
  }
}

function buildState(view: FlowView | null, box?: VideoBox) {
  const win = view?.window ?? null;
  const vb = box ?? { left: 0, top: 0, width: 1, height: 1 };
  const spots: Spot[] = (view?.hotspots ?? []).map((h) => ({
    key: `${h.kind}:${h.id}:${h.choice}`,
    id: h.id,
    kind: h.kind === "inspect" ? "inspect" : "choice",
    shape: h.shape === "circle" ? "circle" : "rect",
    index: h.choice,
    left: vb.left + h.x * vb.width,
    top: vb.top + h.y * vb.height,
    width: h.width * vb.width,
    height: h.height * vb.height,
    label: h.label || view?.choices.find((c) => c.index === h.choice)?.text || h.id,
    inspected: h.inspected,
  }));
  if (!view || !win) {
    return { open: false, hasSpots: spots.length > 0, left: 0, locked: false, held: false, choices: [] as Row[], spots };
  }
  return {
    open: true,
    hasSpots: spots.length > 0,
    left: win.ticksTotal > 0 ? win.ticksLeft / win.ticksTotal : 0,
    locked: win.locked,
    held: win.held,
    choices: view.choices.map((c) => ({
      index: c.index,
      text: c.text,
      spans: c.spans ?? [],
      enabled: c.enabled,
      reason: c.reason ?? "",
      visited: Boolean(c.visited),
      isDefault: win.pending === null && c.index === win.defaultChoice,
      check: c.check ?? null,
      chosen: c.index === win.pending,
      qte: view.qtes.find((q) => q.choice === c.index) ?? null,
    })),
    spots,
  };
}

/**
 * An interactive film's choice window, hotspots, and rich QTE prompts over its
 * footage (flow guide §4):
 * - Choice & non-cutting inspect hotspots align to `videoBox` (landscape or 9:16 portrait),
 *   supporting both `rect` and `circle` shapes.
 * - The countdown bar hides automatically when `window.held` (held or looping segments).
 * - Choice plates surface `reason`, `visited`, `check`, `defaultChoice`, and all QTE
 *   types (`press`, `hold`, `mash`, `timing` zone bar, `sequence`, `swipe`, `drag`).
 */
export function FlowChoices(props: FlowChoicesProps) {
  const now = createMemo(() => buildState(props.view, props.videoBox));
  return (
    <Show when={now().open || now().hasSpots}>
      <View class={`absolute inset-0 pointer-events-none ${packClass("Screen", "frame")}`}>
        <Show when={now().hasSpots}>
          <View class="absolute inset-0 pointer-events-none">
            <For each={now().spots} key={(s) => s.key}>
              {(s) => (
                <View
                  class={`flex-row items-center justify-center gap-1 absolute min-w-11 min-h-11 pointer-events-auto ring-regular ${
                    s().shape === "circle" ? "rounded-full" : "rounded-control"
                  } ${
                    s().kind === "inspect"
                      ? s().inspected
                        ? "ring-fg-muted bg-material-thin opacity-dim hover:bg-material-regular"
                        : "ring-accent bg-material-thin hover:bg-material-regular"
                      : "ring-gold bg-material-thin hover:bg-material-regular disabled:opacity-dim disabled:ring-rim"
                  }`}
                  role="button"
                  label={s().label}
                  data-inspected={s().inspected}
                  disabled={s().kind === "choice" && now().locked}
                  style={{
                    left: `${s().left * 100}%`,
                    top: `${s().top * 100}%`,
                    width: `${s().width * 100}%`,
                    height: `${s().height * 100}%`,
                  }}
                  onPress={(e) => {
                    if (s().kind === "inspect") {
                      props.onInspect?.(e, s().id);
                    } else if (!now().locked && s().index !== null) {
                      props.onChoose?.(e, s().index!);
                    }
                  }}
                >
                  <Show when={s().kind === "inspect" && s().label}>
                    <Text class="px-2 py-0.5 text-xs font-semibold uppercase tracking-wide text-fg bg-surface rounded-control">
                      {s().label}
                    </Text>
                  </Show>
                </View>
              )}
            </For>
          </View>
        </Show>
        <Show when={now().open}>
          <View class="absolute inset-x-0 bottom-0 items-center pointer-events-none bg-linear-to-t from-shade-deep to-transparent pt-12 pb-[env(hud-inset-bottom)] pl-[env(hud-inset-left)] pr-[env(hud-inset-right)]">
            <View class="w-full items-center pb-6 px-12 compact:px-4 compact:pb-4 pointer-events-none">
              <View class="w-full max-w-240 gap-2 pointer-events-none" role="group" label={t("screen.flow-choices.label")}>
                <Show when={!now().held}>
                  <Progress value={now().left * 100} class="h-1 rounded-none" />
                </Show>
                <View class="flex-row flex-wrap gap-2 compact:flex-col compact:flex-nowrap pointer-events-none">
                  <For each={now().choices} key={(c) => c.index}>
                    {(c) => (
                      <View
                        class={`grow shrink basis-42 min-h-14 items-center justify-center gap-1 py-2 px-3 rounded-card pointer-events-auto inset-ring-hairline bg-plate-lo shadow-plate text-fg hover:bg-plate-hi data-chosen:ring-regular data-chosen:ring-accent data-chosen:bg-accent data-chosen:text-accent-fg disabled:opacity-dim compact:basis-auto ${
                          c().isDefault ? "inset-ring-gold" : "inset-ring-border"
                        }`}
                        role="button"
                        current={c().chosen}
                        data-chosen={c().chosen}
                        data-visited={c().visited}
                        disabled={!c().enabled || now().locked}
                        onPress={(e) => {
                          if (c().enabled && !now().locked) props.onChoose?.(e, c().index);
                        }}
                      >
                        <View class="flex-row items-center justify-center gap-2">
                          <Show when={c().qte && c().qte!.action}>
                            <InputGlyph action={c().qte!.action} size="sm" disabled={!c().enabled || now().locked} />
                          </Show>
                          <Show when={c().qte && (c().qte!.kind === "swipe" || c().qte!.kind === "drag")}>
                            <Text class="text-sm font-bold uppercase tracking-wide text-gold">
                              {`${t(qtePromptKey(c().qte!.kind))} ${DIR_ARROW[c().qte!.direction] ?? c().qte!.direction}`}
                            </Text>
                          </Show>
                          <Text class="text-lg font-semibold leading-snug text-center">{c().text}</Text>
                          <Show when={c().check}>
                            <Text class="px-1.5 py-0.5 text-xs font-bold uppercase rounded-control bg-raised-lo text-gold">
                              {`${c().check!.stat} ${Math.round(c().check!.chance * 100)}%`}
                            </Text>
                          </Show>
                          <Show when={c().visited}>
                            <Text class="text-xs font-semibold uppercase tracking-wide text-fg-muted">
                              {t("screen.flow-choices.visited")}
                            </Text>
                          </Show>
                        </View>
                        <Show when={!c().enabled && c().reason}>
                          <Text class="text-xs leading-snug text-center text-fg-muted">{c().reason}</Text>
                        </Show>
                        <Show when={c().qte !== null}>
                          <Show
                            when={c().qte!.kind === "timing"}
                            fallback={
                              <View class="w-full gap-0.5">
                                <Progress
                                  value={c().qte!.progress * 100}
                                  label={t(qtePromptKey(c().qte!.kind))}
                                  class="w-full h-1.5 rounded-none"
                                />
                              </View>
                            }
                          >
                            <View class="relative w-full h-2 rounded-none bg-material-regular overflow-hidden">
                              <View
                                class="absolute inset-y-0 bg-gold"
                                style={{
                                  left: `${c().qte!.zoneStart * 100}%`,
                                  width: `${Math.max(0.04, c().qte!.zoneEnd - c().qte!.zoneStart) * 100}%`,
                                }}
                              />
                              <View
                                class="absolute inset-y-0 w-1 bg-fg"
                                style={{ left: `${c().qte!.progress * 100}%` }}
                              />
                            </View>
                          </Show>
                          <Show when={c().qte!.grade}>
                            <Text class="text-xs font-bold uppercase tracking-wide text-gold">
                              {c().qte!.grade}
                            </Text>
                          </Show>
                        </Show>
                      </View>
                    )}
                  </For>
                </View>
              </View>
            </View>
          </View>
        </Show>
      </View>
    </Show>
  );
}

const view = (pending: number | null, locked = false) => ({
  flow: 1, node: "Kitchen", segment: "segments/kitchen.ogt", texture: 0, time: 9, cut: 14,
  speed: 1, videoWidth: 1920, videoHeight: 1080,
  choices: [
    { index: 0, text: "Take the knife", enabled: true, reason: "", visited: false, spans: [] },
    { index: 1, text: "Open the door", enabled: true, reason: "", visited: true, spans: [] },
    { index: 2, text: "Hold to dodge", enabled: true, reason: "", visited: false, spans: [] },
    { index: 3, text: "Wait in the dark", enabled: false, reason: "Requires key", visited: false, spans: [] },
  ],
  window: { ticksLeft: 180, ticksTotal: 300, defaultChoice: 1, pending, locked, held: false },
  hotspots: [
    { choice: 0, id: "knife", kind: "choice", shape: "rect", x: 0.58, y: 0.3, width: 0.14, height: 0.22, label: "Knife", inspected: false },
    { choice: -1, id: "note", kind: "inspect", shape: "circle", x: 0.25, y: 0.45, width: 0.1, height: 0.1, label: "Read note", inspected: false },
  ],
  qtes: [{ choice: 2, kind: "hold", action: "confirm", direction: "", progress: 0.45, zoneStart: 0, zoneEnd: 0, grade: "", done: false }],
  previous: 0, mix: 1, paused: false, held: false, looping: false, buffering: false, bufferProgress: 1, chapter: "", ending: "", ended: false,
}) as unknown as FlowView;

export const fixtures: Record<string, Partial<FlowChoicesProps>> = {
  default: { view: view(1) },
  locked: { view: view(1, true) },
  closed: { view: null },
};
