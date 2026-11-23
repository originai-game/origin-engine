import { Icon, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface CheckboxProps {
  /** Checked state (or `"mixed"` for indeterminate). */
  checked?: boolean | "mixed" | undefined;
  /** Visible text label beside the box. */
  label?: string | undefined;
  disabled?: boolean | undefined;
  class?: string | undefined;
  tabIndex?: number | undefined;
  autofocus?: boolean | undefined;
  onChange?: ((e: UiEvent, checked: boolean) => void) | undefined;
  [data: `data-${string}`]: unknown;
}

/**
 * A checkbox control with keyboard, pointer, and screen-reader semantics
 * (`role="checkbox"`). Toggling calls `onChange` with the next boolean state.
 */
export function Checkbox(props: CheckboxProps) {
  const on = () => props.checked === true;
  const mixed = () => props.checked === "mixed";
  return (
    <View
      role="checkbox"
      tabIndex={props.tabIndex ?? 0}
      autofocus={props.autofocus}
      disabled={props.disabled}
      checked={props.checked ?? false}
      label={props.label}
      data-checked={on()}
      data-mixed={mixed()}
      onPress={(e) => {
        if (!props.disabled) props.onChange?.(e, !on());
      }}
      class={`group/check flex-row items-center gap-3 min-h-11 py-1 px-2 text-fg disabled:text-fg-subtle disabled:opacity-45 ${props.class ?? ""}`}
    >
      <View
        class={`flex-row items-center justify-center size-5 shrink-0 rounded-control bg-tile inset-ring-1 inset-ring-rim transition duration-200 group-hover/check:bg-surface-hover group-data-checked/check:bg-accent group-data-checked/check:inset-ring-accent ${packClass("Checkbox", "box")}`}
      >
        <Show when={on()}>
          <Icon class="size-3.5 text-accent-fg" name="check" />
        </Show>
        <Show when={mixed()}>
          <View class="w-2.5 h-0.5 rounded-capsule bg-fg" />
        </Show>
      </View>
      <Show when={props.label !== undefined && props.label !== ""}>
        <Text class="text-base text-fg group-disabled/check:text-fg-subtle">{props.label}</Text>
      </Show>
    </View>
  );
}

const noop = () => {};

export const fixtures: Record<string, Partial<CheckboxProps>> = {
  unchecked: { checked: false, label: "Show subtitles", onChange: noop },
  checked: { checked: true, label: "Show subtitles", onChange: noop },
  mixed: { checked: "mixed", label: "Select all", onChange: noop },
  disabled: { checked: true, disabled: true, label: "Locked option", onChange: noop },
};
