import { Icon, Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface CheckboxProps {
  /** Checked or not. The game owns the value: a press reports the next one with `onChange`, and the checkbox shows what the game passes back. */
  checked?: boolean | undefined;
  /** Neither (a "select all" over a partial selection), over `checked`: screen readers say mixed, and a press asks for checked. */
  indeterminate?: boolean | undefined;
  disabled?: boolean | undefined;
  /** Its value fails the game's checks: screen readers say invalid, and `aria-invalid:` styles apply. */
  invalid?: boolean | undefined;
  /** What a screen reader calls it when its children have no text that says it. */
  label?: string | undefined;
  description?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  /** The value a press asks for: `true` from unchecked and from indeterminate, `false` from checked. */
  onChange?: ((e: UiEvent, checked: boolean) => void) | undefined;
  /** The label after the box. */
  children?: any;
}

/**
 * A control that is checked, unchecked or indeterminate (NSH base's
 * `Checkbox`): it owns its presses, keys and state for screen readers;
 * the pack draws its box (`Checkbox.indicator`, whose `checked:` and
 * `indeterminate:` follow it) and the marks inside (`Checkbox.mark`,
 * `Checkbox.dash`).
 */
export function Checkbox(props: CheckboxProps) {
  const state = () => (props.indeterminate ? "mixed" : props.checked === true);
  return (
    <View
      role="checkbox"
      class={`flex-row items-center gap-3 min-h-11 ${props.class ?? ""}`}
      checked={state()}
      disabled={props.disabled}
      invalid={props.invalid}
      label={props.label}
      description={props.description}
      onPress={(e) => props.onChange?.(e, props.indeterminate === true || props.checked !== true)}
    >
      <View class={packClass("Checkbox", "indicator")} checked={state()} disabled={props.disabled} hidden>
        <Show when={props.indeterminate}>
          <View class={packClass("Checkbox", "dash")} />
        </Show>
        <Show when={!props.indeterminate && props.checked}>
          <Icon name="check" class={packClass("Checkbox", "mark")} />
        </Show>
      </View>
      {props.children}
    </View>
  );
}

const change = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<CheckboxProps>> = {
  unchecked: { onChange: change, children: () => <Text>Show damage numbers</Text> },
  checked: { checked: true, onChange: change, children: () => <Text>Vibration</Text> },
  indeterminate: { indeterminate: true, onChange: change, children: () => <Text>All rewards</Text> },
  disabled: { checked: true, disabled: true, onChange: change, children: () => <Text>Cloud saves</Text> },
};
