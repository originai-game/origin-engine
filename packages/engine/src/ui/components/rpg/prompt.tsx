import { Show, Text, View, packClass } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgPromptProps {
  /** The input that does it as the game names it (F, Hold E); empty hides the key cap. */
  key: string;
  /** What it does as the game words it (Finisher: Sunder the guard); empty hides the prompt. */
  text: string;
  /** It cannot be done now: the prompt dims and the reason shows. */
  disabled: boolean;
  /** Why it cannot (Guard holds); shown while disabled. */
  reason: string;
}

/**
 * What the player can do now, over the ability bar: a key cap and the
 * action in words (a finisher, an interaction). The game decides when it
 * shows and what the input does; read only.
 */
export function RpgPrompt(props: RpgPromptProps) {
  return (
    <View class={`group/prompt self-center flex-row items-center justify-center max-w-full gap-1 py-1 px-2 ${packClass("Plate", "frame")}`} role="status" disabled={props.disabled} data-disabled={props.disabled}>
      <Show when={props.text !== ""}>
        <Show when={props.key !== ""}>
          <View class={`shrink min-w-6 max-w-2/5 py-0 px-1 leading-loose text-xs font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key whitespace-nowrap overflow-hidden group-data-disabled/prompt:opacity-dim pointer-coarse:hidden`}>
            <Text>{props.key}</Text>
          </View>
        </Show>
        <Text class={`min-w-0 text-base font-semibold whitespace-nowrap overflow-hidden group-data-disabled/prompt:opacity-dim compact:text-base`}>
          {props.text}
        </Text>
        <Show when={props.disabled && props.reason !== ""}>
          <Text class={`shrink text-xs font-semibold tracking-label uppercase whitespace-nowrap text-danger`}>
            {props.reason}
          </Text>
        </Show>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgPromptProps>> = {
  default: {"key":"F","text":"Finisher: Sunder the guard","disabled":false,"reason":""},
  disabled: {"key":"F","text":"Finisher: Sunder the guard","disabled":true,"reason":"Guard holds"},
  overflow: {"key":"Hold Left Trigger + Right Trigger","text":"Finisher: Sunder the guard of the Ashen Colossus, Warden of the Ninefold Gate","disabled":false,"reason":""},
};
