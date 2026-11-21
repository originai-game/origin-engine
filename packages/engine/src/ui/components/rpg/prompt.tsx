import { Show, Text, View } from "@origingame/engine/ui";
import "./prompt.en.ogt";

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
    <View class={`group/prompt flex-row items-center justify-center max-w-full gap-1 text-fg`} role="status" disabled={props.disabled} data-disabled={props.disabled}>
      <Show when={props.text !== ""}>
        <Show when={props.key !== ""}>
          <View class={`shrink min-w-[1.714rem] max-w-[40%] py-0 px-1 leading-[1.6] text-[0.857rem] font-bold text-center text-[#0b0f18] bg-linear-to-b from-[#ffffff] to-[#c9d1e2] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] whitespace-nowrap overflow-hidden group-data-disabled/prompt:opacity-55 pointer-coarse:hidden`}>
            <Text>{props.key}</Text>
          </View>
        </Show>
        <Text class={`min-w-0 text-[1.214rem] font-semibold whitespace-nowrap overflow-hidden group-data-disabled/prompt:opacity-55 compact:text-[1.143rem]`}>
          {props.text}
        </Text>
        <Show when={props.disabled && props.reason !== ""}>
          <Text class={`shrink text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap text-[#ff4d5e]`}>
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
  empty: {"key":"","text":"","disabled":false,"reason":""},
  overflow: {"key":"Hold Left Trigger + Right Trigger","text":"Finisher: Sunder the guard of the Ashen Colossus, Warden of the Ninefold Gate","disabled":false,"reason":""},
};
