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
        <Text class={`shrink max-w-[8.571rem] min-w-[1.714rem] py-0 px-1 text-[0.857rem] leading-[1.6] font-bold text-center text-[#262b38] bg-linear-to-b from-white to-[#d8dbe2] rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] whitespace-nowrap truncate text-shadow-none`}>
          {props.key}
        </Text>
      </Show>
      <Text class="min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-white/78 whitespace-nowrap truncate text-shadow-2xs">{props.label}</Text>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudHintProps>> = {
  default: {"key":"Tab","label":"Target"},
  empty: {"key":"","label":""},
  overflow: {"key":"Left Shoulder + Right Shoulder","label":"Switch the locked target to the next enemy in sight"},
};
