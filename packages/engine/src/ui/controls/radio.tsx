import { Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface RadioProps {
  /** Checked or not. The game owns the value: a press on an unchecked radio, or the arrows of its group reaching it, asks for `true` with `onChange`. */
  checked?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it when its children have no text that says it. */
  label?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  onChange?: ((e: UiEvent, checked: true) => void) | undefined;
  /** The label after the circle. */
  children?: any;
}

/**
 * One option of a set the player picks one of (NSH base's `Radio`), in a
 * `RadioGroup`: it owns its presses and state for screen readers; a
 * checked radio reports nothing, as it cannot uncheck itself. The pack
 * draws its circle (`Radio.indicator`) and the dot (`Radio.dot`).
 */
export function Radio(props: RadioProps) {
  const ask = (e: UiEvent) => {
    if (props.checked !== true) props.onChange?.(e, true);
  };
  return (
    <View
      role="radio"
      class={`group/radio flex-row items-center gap-3 min-h-11 ${props.class ?? ""}`}
      checked={props.checked === true}
      selected={props.checked === true}
      disabled={props.disabled}
      label={props.label}
      onPress={ask}
    >
      <View class={packClass("Radio", "indicator")} checked={props.checked === true} disabled={props.disabled} hidden>
        <View class={packClass("Radio", "dot")} />
      </View>
      {props.children}
    </View>
  );
}

export interface RadioGroupProps {
  /** What screen readers call the set ("Difficulty"). */
  label?: string | undefined;
  /** Classes of the document that uses it: its layout (`flex-row` for a row). */
  class?: string | undefined;
  children?: any;
}

/**
 * The set around `Radio`s (NSH base's `RadioGroup`): one stop of Tab, at
 * the checked radio; the arrows move among its radios and confirm/press
 * selects the focused radio without trapping gamepad navigation at the
 * boundary. It holds no value: the game keeps which radio is checked.
 */
export function RadioGroup(props: RadioGroupProps) {
  return (
    <View role="radiogroup" label={props.label} focusGroup="stop" focusMemory focusRoving class={`flex-col gap-1 ${props.class ?? ""}`}>
      {props.children}
    </View>
  );
}

const change = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<RadioProps>> = {
  unchecked: { onChange: change, children: () => <Text>Normal</Text> },
  checked: { checked: true, onChange: change, children: () => <Text>Hard</Text> },
  disabled: { disabled: true, onChange: change, children: () => <Text>Nightmare</Text> },
};
