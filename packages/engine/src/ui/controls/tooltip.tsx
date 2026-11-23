import { Show, Text, View, packClass, type UiChild } from "@origingame/engine/ui";

export interface TooltipProps {
  /** Whether the tooltip is visible. */
  open: boolean;
  /** Short explanatory text shown inside the tooltip. */
  text: string;
  /** Optional secondary line under `text`. */
  detail?: string | undefined;
  /** Optional `name` of a UI node to anchor this tooltip to. */
  anchor?: string | undefined;
  /** Placement relative to `anchor` (default `"top"`). */
  side?: "top" | "bottom" | "left" | "right" | undefined;
  class?: string | undefined;
  children?: UiChild;
}

/**
 * A lightweight anchored tooltip (`role="tooltip"`). Set `anchor` to a target
 * node's `name` prop to follow it automatically on the surface.
 */
export function Tooltip(props: TooltipProps) {
  const lift = (): readonly [number, number, number] => {
    switch (props.side ?? "top") {
      case "bottom":
        return [0.5, 1, 0];
      case "left":
        return [0, 0.5, 0];
      case "right":
        return [1, 0.5, 0];
      default:
        return [0.5, 0, 0];
    }
  };
  const offset = () => {
    switch (props.side ?? "top") {
      case "bottom":
        return "-translate-x-1/2 translate-y-2";
      case "left":
        return "-translate-x-full -translate-y-1/2 -ml-2";
      case "right":
        return "-translate-y-1/2 ml-2";
      default:
        return "-translate-x-1/2 -translate-y-full -mt-2";
    }
  };
  return (
    <Show when={props.open}>
      <View
        role="tooltip"
        label={props.text}
        description={props.detail}
        anchor={props.anchor}
        lift={props.anchor ? lift() : undefined}
        class={`pointer-events-none flex-col gap-0.5 max-w-64 py-1.5 px-2.5 rounded-control bg-layer-1 text-fg inset-ring-1 inset-ring-rim shadow-panel ${props.anchor ? offset() : ""} ${packClass("Tooltip", "frame")} ${props.class ?? ""}`}
      >
        <Text class="text-xs font-semibold text-fg leading-snug">{props.text}</Text>
        <Show when={props.detail !== undefined && props.detail !== ""}>
          <Text class="text-xs text-fg-muted leading-snug">{props.detail}</Text>
        </Show>
        {props.children}
      </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<TooltipProps>> = {
  default: { open: true, text: "Fireball", detail: "Deals 35 fire damage in a 3 m radius." },
};
