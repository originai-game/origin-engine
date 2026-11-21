import { Show, Text, View } from "@origingame/engine/ui";
import "./shooter.en.ogt";

export interface ShooterPromptProps {
  /** The input that does it as the game names it (E, X, Hold F); empty hides the key cap. */
  key?: string;
  /** What it does as the game words it (Open supply bin); empty hides the prompt. */
  text: string;
  /** It cannot be done now: the prompt dims and the reason shows. */
  disabled?: boolean;
  /** Why it cannot (Locked); shown while disabled. */
  reason?: string;
}

/**
 * What the player can do with what is in front of them, under the aim: the
 * key cap and the action in words. The game decides when it shows and what
 * the input does; read only. The key cap's scrolling overflow (the svelte's
 * -og-scroll marquee) has no TSX surface: long values truncate.
 */
export function ShooterPrompt(props: ShooterPromptProps) {
  return (
    <View
      class={`group/prompt flex-row items-center justify-center gap-1 max-w-full text-fg data-disabled:opacity-55`}
      role="status"
      data-disabled={props.disabled}
    >
      <Show when={props.text != ""}>
        <Show when={props.key != ""}>
          <Text
            class={`shrink basis-auto max-w-2/5 min-w-5.5 py-0 px-1 text-xs leading-normal font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key whitespace-nowrap truncate`}
          >
            {props.key}
          </Text>
        </Show>
        <Text class={`min-w-0 text-base font-semibold whitespace-nowrap truncate text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-caption`}>{props.text}</Text>
        <Show when={props.disabled && props.reason != ""}>
          <Text class={`shrink-0 text-xs font-semibold tracking-label uppercase text-danger whitespace-nowrap text-shadow-2xs text-stroke-3 text-stroke-bg`}>{props.reason}</Text>
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterPromptProps>> = {
  default: {"key":"E","text":"Open supply bin","disabled":false,"reason":""},
  disabled: {"key":"E","text":"Open armory door","disabled":true,"reason":"Needs a keycard"},
  empty: {"key":"","text":"","disabled":false,"reason":""},
  overflow: {"key":"Hold Left Trigger","text":"Pick up the extended heavy magazine for the Northern Relay marksman rifle","disabled":false,"reason":""},
};
