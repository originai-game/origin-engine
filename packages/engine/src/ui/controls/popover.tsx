import { Show, Text, View, packClass, type UiEvent } from "@origingame/engine/ui";
import { Button } from "./button";

let made = 0;

export interface PopoverProps {
  /** Shown or not. The game owns it: a press on the trigger reports `onOpenChange` with the next value, `back` with false. */
  open: boolean;
  /** Where it stands by its trigger: `bottom` (the default) or `top`. */
  placement?: "bottom" | "top" | undefined;
  /** What screen readers call it. */
  label?: string | undefined;
  /** Classes of the document that uses it, on its panel. */
  class?: string | undefined;
  onOpenChange?: ((e: UiEvent, open: boolean) => void) | undefined;
  /** What opens it: a control its presses go to. */
  trigger?: any;
  children?: any;
}

/**
 * Content by its trigger (NSH base's `Popover`: a filter, a quick
 * detail): the trigger says whether it is expanded, and the panel
 * follows the trigger's box wherever it moves (`anchor`). The pack draws
 * the panel (`Popover.frame`), which pops in by its own `starting:`.
 */
export function Popover(props: PopoverProps) {
  const name = `og-popover-${++made}`;
  const top = () => props.placement === "top";
  return (
    <View class="relative self-start">
      <View name={name} expanded={props.open} onPress={(e) => props.onOpenChange?.(e, !props.open)}>
        {props.trigger}
      </View>
      <Show when={props.open}>
        <View
          role="dialog"
          label={props.label}
          anchor={name}
          lift={[0, top() ? 0 : 1, 0]}
          clamp="edge"
          clampMargin={8}
          class={`z-40 ${top() ? "-translate-y-full -mt-2" : "mt-2"} ${packClass("Popover", "frame")} ${props.class ?? ""}`}
          onAction={{ back: (e) => props.onOpenChange?.(e, false) }}
        >
          {props.children}
        </View>
      </Show>
    </View>
  );
}

const change = () => {};

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<PopoverProps>> = {
  default: {
    open: true,
    label: "Filter",
    onOpenChange: change,
    trigger: () => <Button variant="outline" size="sm"><Text>Filter</Text></Button>,
    children: () => [<Text class="font-semibold">Show</Text>, <Text class="text-fg-muted text-sm">Owned heroes only</Text>],
  },
};
