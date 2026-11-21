import { Show, Text, View } from "@origingame/engine/ui";

export interface HudHintProps {
  /** The input as the game names it (Tab, Space), on a key cap; empty is none. */
  key?: string;
  /** What it does (Target), in small tracked capitals. */
  label?: string;
}

/**
 * One input hint of a HUD, a block a hint row is built from: a key cap
 * and what it does in small tracked capitals. Read only; hidden under a
 * touch screen, where the controls on screen say it. No strings of its
 * own; the svelte file carries none either.
 */
export function HudHint(props: HudHintProps) {
  return (
    <View class={`flex-row items-center gap-1 w-fit min-w-0 max-w-full pointer-coarse:hidden`}>
      <Show when={props.key}>
        <Text class={`shrink max-w-30 min-w-6 py-0 px-1 text-xs leading-loose font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key whitespace-nowrap truncate`}>
          {props.key}
        </Text>
      </Show>
      <Text class="min-w-0 text-xs font-semibold tracking-label uppercase text-fg-muted whitespace-nowrap truncate text-shadow-2xs text-stroke-3 text-stroke-bg">{props.label}</Text>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudHintProps>> = {
  default: {"key":"Tab","label":"Target"},
  empty: {"key":"","label":""},
  overflow: {"key":"Left Shoulder + Right Shoulder","label":"Switch the locked target to the next enemy in sight"},
};
