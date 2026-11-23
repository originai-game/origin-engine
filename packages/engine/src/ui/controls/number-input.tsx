import { Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface NumberInputProps {
  /** The number. The game owns it: a step or a key reports the next one with `onChange`. */
  value: number;
  /** The bounds a step holds it to; none by default. */
  min?: number | undefined;
  max?: number | undefined;
  /** What one step moves it by; default 1. */
  step?: number | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it ("Quantity"). */
  label?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  onChange?: ((e: UiEvent, value: number) => void) | undefined;
}

/**
 * A number stepped up and down (NSH base's `NumberInput`: a quantity to
 * buy, a count to send): a spin button for screen readers whose arrows,
 * Page Up and Down, Home and End step it, between a decrement and an
 * increment button. The pack draws its box (`NumberInput.frame`), its
 * figure (`NumberInput.field`) and its buttons (`NumberInput.step`).
 */
export function NumberInput(props: NumberInputProps) {
  const step = () => props.step ?? 1;
  const hold = (v: number) => Math.min(props.max ?? Infinity, Math.max(props.min ?? -Infinity, v));
  const ask = (e: UiEvent, v: number) => {
    const next = hold(v);
    if (next !== props.value) props.onChange?.(e, next);
  };
  const KEYS: Record<string, (v: number) => number> = {
    ArrowUp: (v) => v + step(),
    ArrowRight: (v) => v + step(),
    ArrowDown: (v) => v - step(),
    ArrowLeft: (v) => v - step(),
    PageUp: (v) => v + step() * 10,
    PageDown: (v) => v - step() * 10,
    Home: (v) => props.min ?? v,
    End: (v) => props.max ?? v,
  };
  return (
    <View class={`${packClass("NumberInput", "frame")} ${props.class ?? ""}`} disabled={props.disabled}>
      <View
        role="button"
        tabIndex={-1}
        class={packClass("NumberInput", "step")}
        label="−"
        disabled={props.disabled || props.value <= (props.min ?? -Infinity)}
        onPress={(e) => ask(e, props.value - step())}
      >
        <Text>−</Text>
      </View>
      <View
        role="spinbutton"
        tabIndex={0}
        class={`flex-row items-center justify-center self-stretch ${packClass("NumberInput", "field")}`}
        disabled={props.disabled}
        label={props.label}
        valueMin={props.min}
        valueMax={props.max}
        valueNow={props.value}
        onKey={(e) => {
          const move = KEYS[e.key];
          if (move && !props.disabled) ask(e, move(props.value));
        }}
      >
        <Text>{String(props.value)}</Text>
      </View>
      <View
        role="button"
        tabIndex={-1}
        class={packClass("NumberInput", "step")}
        label="+"
        disabled={props.disabled || props.value >= (props.max ?? Infinity)}
        onPress={(e) => ask(e, props.value + step())}
      >
        <Text>+</Text>
      </View>
    </View>
  );
}

const change = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<NumberInputProps>> = {
  default: { value: 3, min: 1, max: 99, label: "Quantity", onChange: change },
  "at min": { value: 1, min: 1, max: 99, label: "Quantity", onChange: change },
  disabled: { value: 12, label: "Quantity", disabled: true, onChange: change },
};
