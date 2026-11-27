import { For, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import "./quest-tracker.en.strings.ts";

export interface RpgQuestTrackerProps {
  /** The tracker's heading as the game words it (Side quest); empty takes the component's Main quest. */
  label: string;
  /** The tracked quest's name; empty hides the tracker. */
  name: string;
  /** Its objectives in order, keyed rows of { id, text, done, tag }: done ones are struck and dimmed with a filled diamond; the first not done is the step in play (lit, on data-current); the ones after it are upcoming (dim); tag its progress as the game words it (1 / 3) on a tag, empty for none. A phone shows the step in play alone. */
  steps: ReadonlyArray<{ id: string; text: string; done: boolean; tag?: string }>;
  /** How far its marker is, as the game words it (312 m), on a tag beside the name; empty hides it. */
  distance: string;
}

/**
 * The tracked quest on the HUD: a heading in gold, the quest's name, then
 * its objectives, the done ones struck through, the one in play lit with
 * its progress on a tag and the upcoming ones dim. Read only; the journal
 * owns choosing what to track.
 */
export function RpgQuestTracker(props: RpgQuestTrackerProps) {
  const firstUndone = () => props.steps.findIndex((s) => !s.done);
  return (
    <View class={`gap-y-1.5 gap-x-2 w-62.5 max-w-full px-3 py-2.5 data-empty:hidden compact:w-full ${packClass("Plate", "frame")}`} role="status" label={props.label !== '' ? props.label : t("rpg.quests.main")} data-empty={!props.name}>
      <Show when={props.name}>
        <View class="gap-y-0.5 pb-1 border-b border-border-muted compact:hidden">
          <Text class={`text-xs font-semibold tracking-label uppercase text-gold whitespace-nowrap text-shadow-xs text-shadow-bg overflow-hidden`}>
            {props.label !== '' ? props.label : t("rpg.quests.main")}
          </Text>
          <View class="flex-row items-baseline gap-1.5">
            <Text class={`shrink min-w-0 text-base font-semibold whitespace-nowrap truncate text-fg text-shadow-xs text-shadow-bg`}>
              {props.name}
            </Text>
            <Show when={props.distance}>
              <Text class={`shrink-0 max-w-9/20 text-xs font-bold tabular-nums whitespace-nowrap py-0 px-1.5 overflow-hidden ${packClass("Chip", "frame")}`}>
                {props.distance}
              </Text>
            </Show>
          </View>
        </View>
        <View class="gap-y-1 gap-x-2 p-0 m-0">
          <For each={props.steps}>
            {(s, i) => (
            <View class={`group/step flex-row items-center opacity-muted gap-1.5 compact:hidden data-current:opacity-100 data-done:opacity-dim compact:data-current:flex`} data-done={s().done} data-current={i() === firstUndone()}>
              <View class={`shrink-0 rotate-45 inset-ring-regular inset-ring-gold mx-0.5 size-2 group-data-done/step:bg-none group-data-done/step:bg-gold`} />
              <Text class={`shrink min-w-0 text-caption font-semibold whitespace-nowrap truncate text-fg text-shadow-xs text-shadow-bg group-data-done/step:line-through`}>
                {s().text}
              </Text>
              <Show when={s().tag}>
                <Text class={`shrink-0 max-w-9/20 text-xs font-bold tabular-nums whitespace-nowrap py-0 px-1.5 overflow-hidden ${packClass("Chip", "frame")}`}>
                  {s().tag}
                </Text>
              </Show>
            </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgQuestTrackerProps>> = {
  default: {"label":"","name":"The Ninefold Gate","steps":[{"id":"a","text":"Reach the highland ruins","done":true,"tag":""},{"id":"b","text":"Break the Colossus’s guard","done":false,"tag":"1 / 3"},{"id":"c","text":"Light the three rune pillars","done":false,"tag":""}],"distance":"312 m"},
  done: {"label":"Side quest","name":"The Sunken Bell","steps":[{"id":"a","text":"Speak with the oarsman","done":true,"tag":""},{"id":"b","text":"Find the sunken belfry","done":true,"tag":""}],"distance":""},
  overflow: {"label":"Main quest of the second act of the long campaign","name":"The Ninefold Gate beneath the Ashen Vale and the Drowned Abbey","steps":[{"id":"a","text":"Break the guard of the Ashen Colossus before the ninth bell tolls over the vale","done":false,"tag":"12,345 / 99,999"},{"id":"b","text":"Light the three rune pillars","done":false,"tag":""}],"distance":"12,345 km"},
};
