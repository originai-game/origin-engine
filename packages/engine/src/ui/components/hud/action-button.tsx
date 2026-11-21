import { Icon, Show, Text, View } from "@origingame/engine/ui";

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
      class={`relative rounded-capsule border-2 border-[#8892a8] bg-[#444c5e] shadow-float active:scale-97 disabled:opacity-45 duration-160 data-primary:size-20 data-primary:border-transparent data-primary:bg-[#e8e8ed] size-16`}
      touchButton={props.action}
      data-primary={props.primary}
      disabled={props.disabled}
      label={props.label}
      role="button"
    >
      <View class="size-full flex-row items-center justify-center absolute inset-0 bg-linear-to-b from-white/12 to-black/30 data-primary:from-white/55 data-primary:to-black/10 rounded-capsule" />
      <Show when={(props.charge ?? -1) >= 0}>
        <View class="absolute inset-0 nine-ring bg-gold/50" wipe={Math.min(Math.max(props.charge ?? 0, 0), 1)} data-charged={(props.charge ?? 0) >= 1} />
      </Show>
      <Show when={props.icon} fallback={
        <Text class="text-sm whitespace-nowrap text-fg data-primary:text-[#1a1a1e]">{props.label}</Text>
      }>
        <Icon class="size-8 text-fg data-primary:text-[#1a1a1e]" name={props.icon} />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ActionButtonProps>> = {
  default: {"action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":0.7,"disabled":false},
};
