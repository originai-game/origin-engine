import { Icon, Text, View, createSignal, packClass } from "@origingame/engine/ui";

let made = 0;

export interface TooltipProps {
  /** What it says. */
  text: string;
  /** Where it stands: over its trigger (`top`, the default) or under it (`bottom`). */
  placement?: "top" | "bottom" | undefined;
  /** Shown whatever the pointer does (a tutorial's hint); otherwise it shows while the trigger is hovered or keyboard-focused. */
  open?: boolean | undefined;
  /** Classes of the document that uses it, on its bubble. */
  class?: string | undefined;
  /** The trigger. */
  children?: any;
}

/**
 * A hint over its trigger (NSH base's `Tooltip`): it shows while the
 * pointer is over the trigger or the keyboard's focus is in it
 * (WAI-ARIA's tooltip), dismisses on Escape, and its text describes the
 * trigger for screen readers. The pack draws the bubble (`Tooltip.frame`);
 * it stands centred over the trigger's box wherever it moves (`anchor`).
 */
export function Tooltip(props: TooltipProps) {
  const name = `og-tooltip-${++made}`;
  const dismissed = createSignal(false);
  return (
    <View
      class="group/tip relative self-start"
      name={name}
      description={props.text}
      onKey={(e) => { if (e.key === "Escape") dismissed.set(true); }}
      onBlur={() => dismissed.set(false)}
    >
      {props.children}
      <View
        role="tooltip"
        hidden
        anchor={name}
        lift={[0.5, props.placement === "bottom" ? 1 : 0, 0]}
        clamp="edge"
        clampMargin={8}
        data-open={props.open === true}
        data-dismissed={dismissed() && props.open !== true}
        class={`hidden z-50 pointer-events-none -translate-x-1/2 ${props.placement === "bottom" ? "mt-1.5" : "-translate-y-full -mt-1.5"} group-hover/tip:flex group-focus-within/tip:flex data-open:flex data-dismissed:hidden ${packClass("Tooltip", "frame")} ${props.class ?? ""}`}
      >
        <Text>{props.text}</Text>
      </View>
    </View>
  );
}

/** The gallery mounts it inside a panel, where a game puts it. */
export const ground = "panel";

export const fixtures: Record<string, Partial<TooltipProps>> = {
  default: {
    text: "Critical hits deal double damage",
    open: true,
    placement: "bottom",
    children: () => <Icon name="star" class="size-8" />,
  },
};
