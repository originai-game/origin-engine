import { createSignal, For, Icon, Show, Text, View, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Button } from "../controls/button";
import { Slider } from "../controls/slider";
import { Switch } from "../controls/switch";
import "./hud-editor.en.ogt";

/** One editable HUD element's placement and presentation inside the safe area. */
export interface HudRegion {
  readonly id: string;
  /** Human-readable name shown on the editable tile and in the inspector. */
  readonly label: string;
  /** Horizontal centre across the editable safe area, 0 to 1. */
  readonly x: number;
  /** Vertical centre across the editable safe area, 0 to 1. */
  readonly y: number;
  /** Optional width fraction of the safe area, 0 to 1. */
  readonly width?: number | undefined;
  /** Optional height fraction of the safe area, 0 to 1. */
  readonly height?: number | undefined;
  /** Display scale factor (default 1, inspector range 0.5 to 1.5). */
  readonly scale?: number | undefined;
  /** Display opacity (default 1, inspector range 0.2 to 1). */
  readonly opacity?: number | undefined;
  /** When true, the HUD element is hidden in gameplay and shown dimmed in the editor. */
  readonly hidden?: boolean | undefined;
}

export interface HudEditorScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The hint at the top shows (false once the player has moved something). */
  showHint?: boolean;
  /** Editable HUD regions; when provided, renders interactive draggable boxes and an inspector dock. */
  regions?: ReadonlyArray<HudRegion> | undefined;
  /** Currently selected region id; omit to let the screen track selection locally. */
  selected?: string | undefined;
  /** Draw the safe-area guide frame around the editable stage (default true when regions are given). */
  safeMargin?: boolean | undefined;
  /** Snap step for dragging and keyboard nudging: values >= 1 snap in CSS px; values in (0, 1) snap in normalized fraction; 0 disables snapping. Default 0.025. */
  gridSize?: number | undefined;
  /** Called when a region is selected (or cleared with ""). */
  onSelect?: ((e: UiEvent, id: string) => void) | undefined;
  /** Called whenever a region is dragged, nudged, scaled, faded or toggled. */
  onRegionChange?: ((e: UiEvent, region: HudRegion) => void) | undefined;
  /** Called when the player presses Reset. */
  onReset?: ((e: UiEvent) => void) | undefined;
  /** Called when the player presses Done or the `back` action. */
  onDone?: ((e: UiEvent) => void) | undefined;
  /** Content the caller supplies (was the editorEditor snippet). */
  editorEditor?: (() => JSX.Element) | undefined;
}

function parseBox(value: string): [number, number, number, number] {
  const parts = value.split(" ").map(Number);
  return [parts[0] || 0, parts[1] || 0, parts[2] || 0, parts[3] || 0];
}

function snapCoord(fraction: number, extent: number, grid: number): number {
  const clamped = Math.min(1, Math.max(0, fraction));
  if (grid <= 0) return clamped;
  if (grid >= 1 && extent > 0) {
    return Math.min(1, Math.max(0, (Math.round((clamped * extent) / grid) * grid) / extent));
  }
  if (grid < 1) {
    return Math.min(1, Math.max(0, Math.round(clamped / grid) * grid));
  }
  return clamped;
}

/**
 * The HUD editor's frame: a hint at the top until the player moves something,
 * an interactive safe-area stage with pointer-draggable and keyboard-nudgeable
 * HUD regions, an inspector dock for scale, opacity and visibility, and any
 * custom editor content the caller supplies.
 */
export function HudEditorScreen(props: HudEditorScreenProps) {
  const localSelected = createSignal("");
  const dragging = createSignal<string | null>(null);
  let grabOffsetX = 0;
  let grabOffsetY = 0;

  const regions = () => props.regions ?? [];
  const hasRegions = () => regions().length > 0;
  const grid = () => props.gridSize ?? 0.025;
  const showSafe = () => props.safeMargin ?? hasRegions();
  const activeId = () => (props.selected !== undefined ? props.selected : localSelected());
  const activeRegion = () => regions().find((r) => r.id === activeId());

  const select = (e: UiEvent, id: string) => {
    if (props.selected === undefined) localSelected.set(id);
    props.onSelect?.(e, id);
  };

  const emitChange = (e: UiEvent, region: HudRegion, patch: Partial<HudRegion>) => {
    props.onRegionChange?.(e, { ...region, ...patch });
  };

  const startDrag = (e: UiEvent, region: HudRegion) => {
    select(e, region.id);
    const [bx, by, bw, bh] = parseBox(e.value);
    grabOffsetX = bw > 0 ? e.x - (bx + bw / 2) : 0;
    grabOffsetY = bh > 0 ? e.y - (by + bh / 2) : 0;
    dragging.set(region.id);
  };

  const moveDrag = (e: UiEvent) => {
    const id = dragging();
    if (!id) return;
    const region = regions().find((r) => r.id === id);
    if (!region) return;
    const [sx, sy, sw, sh] = parseBox(e.value);
    if (sw <= 0 || sh <= 0) return;
    const x = snapCoord((e.x - grabOffsetX - sx) / sw, sw, grid());
    const y = snapCoord((e.y - grabOffsetY - sy) / sh, sh, grid());
    emitChange(e, region, { x, y });
  };

  const endDrag = () => {
    dragging.set(null);
  };

  const nudge = (e: UiEvent, region: HudRegion) => {
    const step = grid() > 0 && grid() < 1 ? grid() : 0.02;
    let dx = 0;
    let dy = 0;
    if (e.key === "ArrowLeft") dx = -step;
    else if (e.key === "ArrowRight") dx = step;
    else if (e.key === "ArrowUp") dy = -step;
    else if (e.key === "ArrowDown") dy = step;
    else return;
    emitChange(e, region, {
      x: Math.min(1, Math.max(0, Math.round((region.x + dx) * 1000) / 1000)),
      y: Math.min(1, Math.max(0, Math.round((region.y + dy) * 1000) / 1000)),
    });
  };

  return (
    <View
      class={`absolute inset-0 ${packClass("Screen", "frame")} ${props.class ?? ""}`}
      role="region"
      label={t("screen.hud-editor.stage")}
      onAction={{ back: (e) => props.onDone?.(e) }}
    >
      <Show when={hasRegions() || showSafe()}>
        <View class="absolute inset-0 pt-[env(hud-inset-top)] pb-[env(hud-inset-bottom)] pl-[env(hud-inset-left)] pr-[env(hud-inset-right)] p-4">
          <View
            class="relative size-full"
            onPointerDown={(e) => {
              dragging.set(null);
              select(e, "");
            }}
            onPointerMove={moveDrag}
            onPointerUp={endDrag}
          >
            <Show when={showSafe()}>
              <View
                class="absolute inset-0 pointer-events-none border border-dashed border-border-strong rounded-card"
                label={t("screen.hud-editor.safeArea")}
              />
            </Show>
            <For each={regions()} key={(r) => r.id}>
              {(r) => {
                const isSelected = () => activeId() === r().id;
                const isDragging = () => dragging() === r().id;
                const scale = () => r().scale ?? 1;
                const opacity = () => r().opacity ?? 1;
                return (
                  <View
                    role="button"
                    label={r().label}
                    selected={isSelected()}
                    data-selected={isSelected()}
                    data-dragging={isDragging()}
                    data-hidden={r().hidden === true}
                    class={`absolute -translate-x-1/2 -translate-y-1/2 min-w-28 min-h-11 px-3 py-2 flex-col items-center justify-center gap-0.5 rounded-card bg-glass inset-ring-hairline inset-ring-border transition duration-fast hover:bg-surface-hover focus-visible:inset-ring-regular focus-visible:inset-ring-hud-accent data-selected:inset-ring-regular data-selected:inset-ring-hud-accent data-selected:bg-surface-hover data-selected:shadow-glow data-selected:shadow-hud-accent data-dragging:scale-lift data-hidden:opacity-muted`}
                    style={{
                      left: `${Math.min(1, Math.max(0, r().x)) * 100}%`,
                      top: `${Math.min(1, Math.max(0, r().y)) * 100}%`,
                      width: r().width ? `${r().width! * 100}%` : undefined,
                      height: r().height ? `${r().height! * 100}%` : undefined,
                      transform: `scale(${scale()})`,
                      opacity: r().hidden ? undefined : opacity(),
                    }}
                    onPointerDown={(e) => startDrag(e, r())}
                    onPress={(e) => select(e, r().id)}
                    onKey={(e) => nudge(e, r())}
                  >
                    <View class="flex-row items-center gap-1.5 min-w-0">
                      <Show when={r().hidden}>
                        <Icon name="eyeOff" class="size-3.5 text-fg-muted shrink-0" />
                      </Show>
                      <Text class="text-caption font-semibold uppercase tracking-label text-fg whitespace-nowrap truncate">
                        {r().label}
                      </Text>
                    </View>
                    <Text class="text-2xs tabular-nums text-fg-muted whitespace-nowrap">
                      {r().hidden
                        ? t("screen.hud-editor.hidden")
                        : `${formatNumber(Math.round(scale() * 100))}% · ${formatNumber(Math.round(opacity() * 100))}%`}
                    </Text>
                  </View>
                );
              }}
            </For>
          </View>
        </View>
      </Show>
      <Show when={props.showHint}>
        <View class={`absolute w-max top-6 bg-plate-hi backdrop-blur-regular py-3 px-6 mx-auto inset-x-0 compact:top-12 ${packClass("Panel", "frame")}`}>
          <Text class={`text-center line-clamp-2 text-reading font-semibold text-fg m-0`}>
            {t("screen.hud-editor.hint")}
          </Text>
        </View>
      </Show>
      <View class={`absolute w-max max-w-full bottom-6 mx-auto inset-x-0 flex-col items-center gap-2 px-4 narrow:left-4 narrow:right-0 narrow:w-auto narrow:bottom-8 narrow:mx-0 short:left-24 short:right-0 short:w-auto short:mx-0`}>
        <Show when={hasRegions()}>
          <View class={`flex-row flex-wrap items-center justify-center gap-x-4 gap-y-2 py-2.5 px-4 bg-plate-hi backdrop-blur-regular ${packClass("Panel", "frame")}`}>
            <Show
              when={activeRegion()}
              fallback={
                <Text class="text-caption text-fg-muted whitespace-nowrap">
                  {t("screen.hud-editor.none")}
                </Text>
              }
            >
              {(selected) => (
                <View class="flex-row flex-wrap items-center gap-x-4 gap-y-2">
                  <Text class="text-caption font-bold uppercase tracking-label text-hud-accent-ink whitespace-nowrap">
                    {selected().label}
                  </Text>
                  <View class="flex-row items-center gap-2">
                    <Text class="text-xs font-semibold text-fg-muted whitespace-nowrap">{t("screen.hud-editor.scale")}</Text>
                    <Slider
                      class="w-28 compact:w-20"
                      label={t("screen.hud-editor.scale")}
                      value={selected().scale ?? 1}
                      min={0.5}
                      max={1.5}
                      step={0.05}
                      valueText={`${formatNumber(Math.round((selected().scale ?? 1) * 100))}%`}
                      onChange={(e, v) => emitChange(e, selected(), { scale: Math.round(v * 100) / 100 })}
                    />
                    <Text class="text-xs tabular-nums text-fg min-w-10 text-end">
                      {`${formatNumber(Math.round((selected().scale ?? 1) * 100))}%`}
                    </Text>
                  </View>
                  <View class="flex-row items-center gap-2">
                    <Text class="text-xs font-semibold text-fg-muted whitespace-nowrap">{t("screen.hud-editor.opacity")}</Text>
                    <Slider
                      class="w-28 compact:w-20"
                      label={t("screen.hud-editor.opacity")}
                      value={selected().opacity ?? 1}
                      min={0.2}
                      max={1}
                      step={0.05}
                      valueText={`${formatNumber(Math.round((selected().opacity ?? 1) * 100))}%`}
                      onChange={(e, v) => emitChange(e, selected(), { opacity: Math.round(v * 100) / 100 })}
                    />
                    <Text class="text-xs tabular-nums text-fg min-w-10 text-end">
                      {`${formatNumber(Math.round((selected().opacity ?? 1) * 100))}%`}
                    </Text>
                  </View>
                  <Switch
                    class="min-h-11"
                    label={t("screen.hud-editor.visible")}
                    checked={selected().hidden !== true}
                    onChange={(e, on) => emitChange(e, selected(), { hidden: !on })}
                  >
                    <Text class="text-xs font-semibold text-fg-muted whitespace-nowrap">{t("screen.hud-editor.visible")}</Text>
                  </Switch>
                </View>
              )}
            </Show>
            <View class="flex-row items-center gap-2">
              <Show when={props.onReset}>
                <Button size="sm" variant="ghost" onPress={(e) => props.onReset?.(e)}>
                  <Text>{t("screen.hud-editor.reset")}</Text>
                </Button>
              </Show>
              <Show when={props.onDone}>
                <Button size="sm" variant="primary" onPress={(e) => props.onDone?.(e)}>
                  <Text>{t("screen.hud-editor.done")}</Text>
                </Button>
              </Show>
            </View>
          </View>
        </Show>
        {props.editorEditor?.()}
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudEditorScreenProps>> = {
  default: {"showHint":true},
  regions: {
    "showHint": true,
    "selected": "vitals",
    "regions": [
      {"id":"vitals","label":"Vitals","x":0.14,"y":0.86,"scale":1,"opacity":1,"hidden":false},
      {"id":"minimap","label":"Minimap","x":0.88,"y":0.16,"scale":1,"opacity":0.95,"hidden":false},
      {"id":"objectives","label":"Objectives","x":0.14,"y":0.16,"scale":0.95,"opacity":0.9,"hidden":false},
      {"id":"abilities","label":"Ability Bar","x":0.5,"y":0.88,"scale":1.05,"opacity":1,"hidden":false},
    ],
  },
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Done"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"hud-editor","layer":"modal","focus":"","under":"none","input":"menu","pause":true,"capture":true,"remember":true,"back":"back"} as const;
