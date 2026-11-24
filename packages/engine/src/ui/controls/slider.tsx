import { View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface SliderProps {
  /** The value, from `min` to `max`. The game owns it: a press, a drag or a key reports the next one with `onChange`. */
  value: number;
  /** The range; default 0 to 100. */
  min?: number | undefined;
  max?: number | undefined;
  /** What the value snaps to, and what one arrow key moves it by; default 1. */
  step?: number | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it ("Music volume"). */
  label?: string | undefined;
  /** The value as words for screen readers ("80%"). */
  valueText?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  /** The value it asks for, snapped to the step, as often as it changes during a drag. */
  onChange?: ((e: UiEvent, value: number) => void) | undefined;
  /** The change ended (the pointer lifted, a key's step done): the value it ended at, once. */
  onChangeEnd?: ((e: UiEvent, value: number) => void) | undefined;
}

/**
 * A value picked along a track (NSH base's `Slider`: a volume, a
 * sensitivity): it owns its presses, drags, keys (the arrows by a step,
 * Page Up and Down by a tenth, Home and End to the ends) and its range for
 * screen readers; the pack draws its track (`Slider.track`), the filled
 * range (`Slider.range`) and the thumb (`Slider.thumb`).
 */
export function Slider(props: SliderProps) {
  const min = () => (Number.isFinite(props.min) ? props.min! : 0);
  const max = () => (Number.isFinite(props.max) ? props.max! : 100);
  const step = () => {
    const s = props.step ?? 1;
    return Number.isFinite(s) && s > 0 ? s : 1;
  };
  const current = () => (Number.isFinite(props.value) ? props.value : min());
  const clamp = (v: number) => {
    const lo = min();
    const hi = max();
    const s = step();
    const raw = Number.isFinite(v) ? v : lo;
    return Math.min(hi, Math.max(lo, lo + Math.round((raw - lo) / s) * s));
  };
  const share = () => (max() > min() ? (Math.min(max(), Math.max(min(), current())) - min()) / (max() - min()) : 0);
  let dragging = false;
  let last = current();
  const ask = (e: UiEvent, v: number) => {
    last = clamp(v);
    if (last !== current()) props.onChange?.(e, last);
  };
  const drag = (e: UiEvent) => {
    if (props.disabled) return;
    const [bx = 0, , bw = 0] = e.value.split(" ").map(Number);
    ask(e, min() + Math.min(1, Math.max(0, (e.x - bx) / (bw || 1))) * (max() - min()));
  };
  const KEYS: Record<string, (v: number) => number> = {
    ArrowRight: (v) => v + step(),
    ArrowLeft: (v) => v - step(),
    PageUp: (v) => v + Math.max(step(), (max() - min()) / 10),
    PageDown: (v) => v - Math.max(step(), (max() - min()) / 10),
    Home: () => min(),
    End: () => max(),
  };
  return (
    <View
      role="slider"
      tabIndex={0}
      class={`group/slider flex-row items-center min-h-11 ${props.class ?? ""}`}
      disabled={props.disabled}
      label={props.label}
      valueMin={min()}
      valueMax={max()}
      valueNow={current()}
      valueText={props.valueText}
      onPointerDown={(e) => {
        if (props.disabled) return;
        dragging = true;
        last = current();
        drag(e);
      }}
      onPointerMove={(e) => {
        if (dragging) drag(e);
      }}
      onPointerUp={(e) => {
        if (!dragging) return;
        dragging = false;
        props.onChangeEnd?.(e, last);
      }}
      onKey={(e) => {
        const move = KEYS[e.key];
        if (!move || props.disabled) return;
        last = current();
        ask(e, move(current()));
        props.onChangeEnd?.(e, last);
      }}
    >
      <View class={packClass("Slider", "track")} disabled={props.disabled}>
        <View class={packClass("Slider", "range")} style={{ width: `${share() * 100}%` }} />
        <View class={packClass("Slider", "thumb")} disabled={props.disabled} style={{ left: `${share() * 100}%` }} />
      </View>
    </View>
  );
}

const change = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<SliderProps>> = {
  default: { value: 70, label: "Music volume", onChange: change },
  empty: { value: 0, label: "Shake", onChange: change },
  full: { value: 100, label: "Brightness", onChange: change },
  disabled: { value: 40, label: "Voice volume", disabled: true, onChange: change },
};
