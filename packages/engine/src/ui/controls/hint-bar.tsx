import { For, Text, View, packClass } from "@origingame/engine/ui";
import { InputGlyph } from "./input-glyph";

export interface HintBarProps {
  /** The hints, in order: each an input action (confirm, back, tabNext) and the words for what it does here. */
  hints: ReadonlyArray<{ action: string; label: string }>;
  /** Classes after its own (its placement). */
  class?: string | undefined;
}

/**
 * The footer of input hints a full-screen built-in shows (A Select ·
 * B Back): each action's glyph on the device used last, then its words, in
 * the pack's hint type (Hint.bar, Hint.item). Hidden on touch, where no
 * button is pressed. Purely presentational.
 */
export function HintBar(props: HintBarProps) {
  return (
    <View class={`pointer-events-none pointer-coarse:hidden ${packClass("Hint", "bar")} ${props.class ?? ""}`}>
      <For each={props.hints}>
        {(h) => (
          <View class={packClass("Hint", "item")}>
            <InputGlyph action={h().action} size="lg" />
            <Text class="m-0 whitespace-nowrap">{h().label}</Text>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<HintBarProps>> = {
  default: { hints: [{ action: "confirm", label: "Select" }, { action: "back", label: "Back" }] },
};
