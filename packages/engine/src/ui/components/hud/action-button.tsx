import { Icon, Show, Text, View, packClass } from "@origingame/engine/ui";

export interface ActionButtonProps {
  /** The og-input action it feeds while a touch holds it (jump). */
  action: string;
  /** Its words, its accessible name; shown when it has no icon. */
  label: string;
  /** An icon of the atlas on its face; empty shows the label. */
  icon?: string;
  /** The main action: the primary fill and a larger disc. */
  primary?: boolean;
  /** A charge round its rim, 0 to 1 (an ultimate); negative is none. Default -1. */
  charge?: number;
  /** It feeds nothing and shows faded. */
  disabled?: boolean;
}

/**
 * A touch disc: an action a finger holds, the primary one larger, an
 * optional charge ring round its rim. The disc's box is a control zone of
 * its og-input action (touchButton), so the action is 1 while a touch
 * that started on it is held; the disc has no press handler of its own.
 * The face's radial sheen is a linear one.
 */
export function ActionButton(props: ActionButtonProps) {
  return (
    <View
      class={`flex-row items-center justify-center gap-1 relative group/disc ${packClass("Touch", "button")} ${props.primary ? packClass("Touch", "button", "primary") : ""}`}
      touchButton={props.action}
      data-primary={props.primary}
      disabled={props.disabled}
      label={props.label}
      role="button"
    >
      <View class="size-full flex-row items-center justify-center absolute inset-0 bg-linear-to-b from-lit to-shade group-data-primary/disc:from-sheen group-data-primary/disc:to-shade rounded-capsule" />
      <Show when={(props.charge ?? -1) >= 0}>
        <View class={`absolute inset-0 ${packClass("Touch", "charge")} ${(props.charge ?? 0) >= 1 && !props.disabled ? packClass("Touch", "charge", "ready") : ""}`} wipe={Math.min(Math.max(props.charge ?? 0, 0), 1)} />
      </Show>
      <Show when={props.icon} fallback={
        <Text class="text-sm whitespace-nowrap">{props.label}</Text>
      }>
        <Icon class={packClass("Touch", "icon")} name={props.icon} />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ActionButtonProps>> = {
  default: {"action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":0.7,"disabled":false},
};
