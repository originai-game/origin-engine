import { Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface SwitchProps {
  /** On or off. The game owns the value: a press reports the opposite with `onChange`. */
  checked?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it when its children have no text that says it. */
  label?: string | undefined;
  description?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  onChange?: ((e: UiEvent, checked: boolean) => void) | undefined;
  /** The label before the track. */
  children?: any;
}

/**
 * A setting that is on or off and takes effect at once (NSH base's
 * `Switch`): it owns its presses, keys and state for screen readers; the
 * pack draws its track (`Switch.track`) and the thumb that moves along it
 * (`Switch.thumb`), whose `checked:` and `disabled:` follow the switch.
 */
export function Switch(props: SwitchProps) {
  return (
    <View
      role="switch"
      haptic="tick"
      class={`group/switch flex-row items-center justify-between gap-3 min-h-11 ${props.class ?? ""}`}
      checked={props.checked === true}
      disabled={props.disabled}
      label={props.label}
      description={props.description}
      onPress={(e) => props.onChange?.(e, props.checked !== true)}
    >
      {props.children}
      <View class={packClass("Switch", "track")} checked={props.checked === true} disabled={props.disabled} hidden>
        <View class={packClass("Switch", "thumb")} checked={props.checked === true} disabled={props.disabled} />
      </View>
    </View>
  );
}

const change = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<SwitchProps>> = {
  off: { onChange: change, children: () => <Text>Music</Text> },
  on: { checked: true, onChange: change, children: () => <Text>Sound effects</Text> },
  disabled: { checked: true, disabled: true, onChange: change, children: () => <Text>Cross-play</Text> },
};
