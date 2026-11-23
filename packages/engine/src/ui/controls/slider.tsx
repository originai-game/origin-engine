import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface SliderProps {
  /** Current value within `[min, max]`. */
  value: number;
  min?: number | undefined;
  max?: number | undefined;
  step?: number | undefined;
  /** Accessible name (and optional displayed label). */
  label?: string | undefined;
  /** Show the formatted numeric value beside the track. */
  showValue?: boolean | undefined;
  disabled?: boolean | undefined;
  class?: string | undefined;
  tabIndex?: number | undefined;
  onChange?: ((e: UiEvent, value: number) => void) | undefined;
  [data: `data-${string}`]: unknown;
}

/**
 * A range slider (`role="slider"`): supports pointer drag along the track and
 * Left/Right/Up/Down/Home/End keys.
 */
export function Slider(props: SliderProps) {
  const lo = () => props.min ?? 0;
  const hi = () => props.max ?? 100;
  const step = () => (props.step && props.step > 0 ? props.step : 1);
  const clamped = () => Math.min(hi(), Math.max(lo(), props.value));
  const pct = () => {
    const span = hi() - lo();
    return span <= 0 ? 0 : Math.max(0, Math.min(100, ((clamped() - lo()) / span) * 100));
  };
  const snap = (raw: number) => {
    const s = step();
    const snapped = lo() + Math.round((raw - lo()) / s) * s;
    return Math.min(hi(), Math.max(lo(), Number(snapped.toFixed(6))));
  };
  const dragged = (e: UiEvent) => {
    if (props.disabled) return;
    const [bx, , bw] = e.value.split(" ").map(Number);
    if (!Number.isFinite(bx) || !Number.isFinite(bw) || bw <= 0) return;
    const t = Math.min(1, Math.max(0, (e.x - bx) / bw));
    props.onChange?.(e, snap(lo() + t * (hi() - lo())));
  };
  return (
    <View
      role="slider"
      tabIndex={props.tabIndex ?? 0}
      disabled={props.disabled}
      label={props.label}
      valueMin={lo()}
      valueMax={hi()}
      valueNow={clamped()}
      onKey={(e) => {
        if (props.disabled) return;
        const s = step();
        if (e.key === "ArrowRight" || e.key === "ArrowUp") props.onChange?.(e, snap(clamped() + s));
        else if (e.key === "ArrowLeft" || e.key === "ArrowDown") props.onChange?.(e, snap(clamped() - s));
        else if (e.key === "Home") props.onChange?.(e, lo());
        else if (e.key === "End") props.onChange?.(e, hi());
      }}
      class={`flex-row items-center gap-3 min-h-11 disabled:opacity-45 ${props.class ?? ""}`}
    >
      <Show when={props.label !== undefined && props.label !== ""}>
        <Text class="text-base text-fg-muted shrink-0">{props.label}</Text>
      </Show>
      <View
        class={`relative grow shrink basis-0 h-2 rounded-capsule bg-track inset-shadow-lit inset-ring-1 inset-ring-rim ${packClass("Slider", "track")}`}
        onPointerDown={dragged}
        onPointerMove={dragged}
      >
        <View
          class={`absolute left-0 top-0 h-full rounded-capsule bg-accent ${packClass("Slider", "fill")}`}
          style={{ width: `${pct()}%` }}
        />
        <View
          class={`absolute top-1/2 size-4 -translate-y-1/2 rounded-capsule bg-accent shadow-panel ${packClass("Slider", "thumb")}`}
          style={{ left: `calc(${pct()}% - 8px)` }}
        />
      </View>
      <Show when={props.showValue}>
        <Text class="text-sm tabular-nums text-fg min-w-10 text-end shrink-0">{String(clamped())}</Text>
      </Show>
    </View>
  );
}

const noop = () => {};

export const fixtures: Record<string, Partial<SliderProps>> = {
  default: { value: 65, min: 0, max: 100, step: 5, label: "Volume", showValue: true, onChange: noop },
  min: { value: 0, min: 0, max: 100, label: "Music", showValue: true, onChange: noop },
  max: { value: 100, min: 0, max: 100, label: "SFX", showValue: true, onChange: noop },
  disabled: { value: 40, min: 0, max: 100, disabled: true, label: "Locked", showValue: true, onChange: noop },
};
