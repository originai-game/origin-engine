import { Text, View, packClass } from "@origingame/engine/ui";
import { Button } from "./button";
import { Toggle } from "./toggle";

export interface ToolbarProps {
  /** What screen readers call the toolbar ("Formatting"), when a screen has more than one. */
  label?: string | undefined;
  /** Classes of the document that uses it, after its own. */
  class?: string | undefined;
  /** Its `Button`s, `Toggle`s, `ToggleGroup`s and `ToolbarSeparator`s. */
  children?: any;
}

/**
 * A bar of controls (NSH base's `Toolbar`: a player's transport, a palette
 * of tools): one stop of Tab, at the item the focus last left; the arrows
 * move among its items without pressing them, a `ToggleGroup`'s inside it
 * included (WAI-ARIA's toolbar). The pack draws its bar (`Toolbar.frame`).
 */
export function Toolbar(props: ToolbarProps) {
  return (
    <View role="toolbar" label={props.label} focusGroup="stop" focusMemory focusRoving class={`${packClass("Toolbar", "frame")} ${props.class ?? ""}`}>
      {props.children}
    </View>
  );
}

/** A line between runs of a toolbar's items (`Toolbar.separator`), never a focus stop. */
export function ToolbarSeparator(props: { class?: string | undefined }) {
  return <View role="separator" class={`${packClass("Toolbar", "separator")} ${props.class ?? ""}`} />;
}

const press = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<ToolbarProps>> = {
  default: {
    label: "Replay",
    children: () => [
      <Button variant="ghost" size="sm" onPress={press}><Text>Rewind</Text></Button>,
      <Button variant="ghost" size="sm" onPress={press}><Text>Pause</Text></Button>,
      <ToolbarSeparator />,
      <Toggle pressed onChange={press}><Text>Slow</Text></Toggle>,
    ],
  },
};
