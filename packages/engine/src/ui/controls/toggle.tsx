import { Text, View, packClass, type UiEvent } from "@origingame/engine/ui";

export interface ToggleGroupProps {
  /** What screen readers call the group ("View"). */
  label?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  children?: any;
}

/**
 * A row of `Toggle`s (NSH base's `ToggleGroup`); one pressed at a time is
 * rune-kit's segmented control, which the game keeps by pressing the one
 * asked for and letting go of the others. One stop of Tab, at its first
 * pressed toggle; the arrows move among its toggles without pressing them
 * (WAI-ARIA's toolbar). The pack draws its well (`ToggleGroup.frame`).
 */
export function ToggleGroup(props: ToggleGroupProps) {
  return (
    <View role="toolbar" label={props.label} focusGroup="wrap" focusRoving class={`${packClass("ToggleGroup", "frame")} ${props.class ?? ""}`}>
      {props.children}
    </View>
  );
}

export interface ToggleProps {
  /** On or off. The game owns the value: a press reports the opposite with `onChange`. */
  pressed?: boolean | undefined;
  disabled?: boolean | undefined;
  /** What a screen reader calls it when its children have no text that says it (an icon). */
  label?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  onChange?: ((e: UiEvent, pressed: boolean) => void) | undefined;
  children?: any;
}

/**
 * A button that is on or off (NSH base's `Toggle`), alone or in a
 * `ToggleGroup`: it owns its presses and its pressed state for screen
 * readers; the pack dresses it (`Toggle.item`, `aria-pressed:` while on).
 */
export function Toggle(props: ToggleProps) {
  return (
    <View
      role="button"
      class={`${packClass("Toggle", "item")} ${props.class ?? ""}`}
      pressed={props.pressed === true}
      disabled={props.disabled}
      label={props.label}
      onPress={(e) => props.onChange?.(e, props.pressed !== true)}
    >
      {props.children}
    </View>
  );
}

const change = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<ToggleGroupProps>> = {
  segmented: {
    label: "Range",
    children: () => [
      <Toggle class="grow" pressed onChange={change}><Text>Day</Text></Toggle>,
      <Toggle class="grow" onChange={change}><Text>Week</Text></Toggle>,
      <Toggle class="grow" onChange={change}><Text>All time</Text></Toggle>,
    ],
  },
};
