import { For, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./shooter.en.ogt";

export interface ShooterGoal {
  id: string;
  /** The objective as the game words it. */
  text: string;
  /** Its progress or time on a tag (2 / 3, 0:45), empty for none. */
  tag?: string;
  /** The one in play (lit; the others are dimmed). */
  current?: boolean;
}

export interface ShooterObjectivesProps {
  /** The block's heading as the game words it; empty takes the component's Objective. */
  label?: string;
  /**
   * The objectives in the game's order. The one in play is lit with its
   * progress on a tag, the others dimmed under it with theirs (a phone
   * shows the one in play alone). The game owns progress and completion.
   * Read only.
   */
  goals?: ReadonlyArray<ShooterGoal>;
}

/**
 * The objectives at the screen's top corner: a heading in the HUD's accent,
 * the objective in play lit with its progress on a tag, the others dimmed
 * under it with theirs (a phone shows the one in play alone). Read only.
 * The tags' scrolling overflow (the svelte's -og-scroll marquee) has no TSX
 * surface: long values truncate.
 */
export function ShooterObjectives(props: ShooterObjectivesProps) {
  const heading = () => (props.label ?? "") != "" ? props.label : t("shooter.objective.label");
  return (
    <View
      class={`flex-col gap-y-[0.429rem] w-[20rem] max-w-full compact:w-full text-fg`}
      role="group"
      label={heading()}
    >
      <Show when={(props.goals ?? []).length}>
        <Text class={`min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap truncate text-[#38e8ff] text-shadow-2xs text-stroke-3 text-stroke-bg compact:hidden`}>{heading()}</Text>
        <For each={props.goals ?? []} key={(g) => g.id}>
          {(g) => (
            <View class={`flex-row items-center gap-1 min-w-0 opacity-66 data-current:opacity-100 compact:hidden data-current:compact:flex`}>
              <View
                class={`shrink-0 size-[0.571rem] mx-[0.143rem] rotate-45 bg-white/66 shadow-[0px_0px_0px_0.143rem_#1a1e28d9] data-current:size-[0.714rem] data-current:mx-[0.071rem] data-current:bg-gold`}
              />
              <Text
                class={`shrink basis-auto min-w-0 text-[1.071rem] font-semibold whitespace-nowrap truncate text-fg text-shadow-2xs text-stroke-3 text-stroke-bg data-current:text-[1.143rem]`}
              >
                {g().text}
              </Text>
              <Show when={g().tag}>
                <Text class={`shrink-0 py-0 px-1 text-[0.857rem] font-bold tabular-nums whitespace-nowrap text-fg nine-chip-cut bg-tile ${packClass("Chip", "frame")}`}>
                  {g().tag}
                </Text>
              </Show>
            </View>
          )}
        </For>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterObjectivesProps>> = {
  default: {"label":"","goals":[{"id":"relay","text":"Secure the relay tower","tag":"2 / 3","current":true},{"id":"catwalk","text":"Hold the catwalk","tag":"0:45","current":false}]},
  one: {"label":"Contract","goals":[{"id":"extract","text":"Reach the extraction point","tag":"","current":true}]},
  empty: {"label":"","goals":[]},
  overflow: {"label":"Primary and secondary objectives of the operation","goals":[{"id":"long","text":"Secure the relay tower on the northern ridge before the storm wall closes in","tag":"12,345 / 99,999","current":true},{"id":"two","text":"Hold the catwalk above the old refinery until reinforcements arrive","tag":"1:02:45","current":false},{"id":"three","text":"Recover the black box","tag":"","current":false}]},
};
