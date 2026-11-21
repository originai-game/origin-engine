import { For, Show, Text, View, t } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgHintsProps {
  /** The inputs worth a reminder, keyed rows of { id, key, label }: key the input as the game names it (Tab, Space) on a key cap, label what it does (Target). Under a touch screen the hints are hidden: the controls on screen say it. */
  hints?: ReadonlyArray<{ id: string; key: string; label: string }>;
}

/**
 * Input hints at the screen's foot: a row of key caps, each with what it
 * does in small tracked capitals. The row only spaces them; a game lays
 * a hint out anywhere else itself. Read only; hidden under a touch
 * screen.
 */
export function RpgHints(props: RpgHintsProps) {
  return (
    <View
      class={`flex-row flex-wrap justify-end max-w-full text-fg border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border p-2 gap-x-[1.143rem] gap-y-1 pointer-coarse:hidden`}
      role="list"
      label={t("rpg.hints.name")}
    >
      <For each={props.hints ?? []} key={(h) => h.id}>
        {(h) => (
        <View class={`flex-row items-center min-w-0 max-w-full gap-1`} role="listitem">
          <Show when={h().key}>
            <Text
              class={`shrink min-w-[1.714rem] max-w-[8.571rem] px-1 text-[0.857rem] leading-[1.6] font-bold text-center text-[#0b0f18] bg-linear-to-b from-[#ffffff] to-[#c9d1e2] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] whitespace-nowrap overflow-hidden text-ellipsis`}
            >
              {h().key}
            </Text>
          </Show>
          <Text
            class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-[#ffffffc7] whitespace-nowrap overflow-hidden text-ellipsis text-stroke-3 text-stroke-bg`}
          >
            {h().label}
          </Text>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgHintsProps>> = {
  default: {"hints":[{"id":"target","key":"Tab","label":"Target"},{"id":"dodge","key":"Space","label":"Dodge"}]},
  empty: {"hints":[]},
  overflow: {"hints":[{"id":"a","key":"Left Shoulder + Right Shoulder","label":"Switch the locked target to the next enemy in sight"},{"id":"b","key":"B","label":"Dodge"}]},
};
