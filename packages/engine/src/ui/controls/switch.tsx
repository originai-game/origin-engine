import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface SwitchProps {
  /** Whether the switch is on. */
  checked?: boolean | undefined;
  /** Visible text label beside the track. */
  label?: string | undefined;
  disabled?: boolean | undefined;
  class?: string | undefined;
  tabIndex?: number | undefined;
  autofocus?: boolean | undefined;
  onChange?: ((e: UiEvent, checked: boolean) => void) | undefined;
  [data: `data-${string}`]: unknown;
}

/**
 * A binary toggle switch (`role="switch"`). Clicking or activating it calls
 * `onChange` with the toggled boolean value.
 */
export function Switch(props: SwitchProps) {
  const on = () => props.checked === true;
  return (
    <View
      role="switch"
      tabIndex={props.tabIndex ?? 0}
      autofocus={props.autofocus}
      disabled={props.disabled}
      checked={on()}
      label={props.label}
      data-checked={on()}
      onPress={(e) => {
        if (!props.disabled) props.onChange?.(e, !on());
      }}
      class={`group/switch flex-row items-center gap-3 min-h-11 py-1 px-2 text-fg disabled:text-fg-subtle disabled:opacity-45 ${props.class ?? ""}`}
    >
      <Show when={props.label !== undefined && props.label !== ""}>
        <Text class="text-base text-fg grow shrink">{props.label}</Text>
      </Show>
      <View
        class={`relative h-5 w-9 shrink-0 rounded-capsule bg-track inset-ring-1 inset-ring-rim transition duration-200 group-data-checked/switch:bg-surface-hover group-data-checked/switch:inset-ring-accent ${packClass("Switch", "track")}`}
      >
        <View
          class={`absolute top-0.5 left-0.5 size-4 rounded-capsule bg-fg-muted transition-transform duration-200 ease-out group-data-checked/switch:translate-x-4 group-data-checked/switch:bg-accent ${packClass("Switch", "thumb")}`}
        />
      </View>
    </View>
  );
}

const noop = () => {};

export const fixtures: Record<string, Partial<SwitchProps>> = {
  off: { checked: false, label: "Screen shake", onChange: noop },
  on: { checked: true, label: "Screen shake", onChange: noop },
  disabled: { checked: true, disabled: true, label: "V-Sync", onChange: noop },
};
