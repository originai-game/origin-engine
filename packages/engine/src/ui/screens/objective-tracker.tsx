import {For, Show, Text, View, packClass} from "@origingame/engine/ui";

export interface ObjectiveTrackerProps {
  /** The quest or chapter. */
  title: string;
  /** A keyed collection of { id, label, progress (0–1), done, steps }: steps, when above 1, cuts the bar into that many cells (2 / 5 is five); 0 or absent is one bar. */
  objectives: ReadonlyArray<{ id: string; label: string; progress: number; done: boolean; steps: number }>;
}

/**
 * The tracked objectives over the scene, with no plate: the quest in gold
 * small capitals, then each objective with a gold diamond (filled once
 * done, its words struck and dimmed) over a thin leaning bar in a dark
 * well, cut into cells when the game counts its steps. The first not done
 * is lit; on a phone only it is shown. Read only.
 */
export function ObjectiveTracker(props: ObjectiveTrackerProps) {
  return (
    <View class={`text-fg w-75 max-w-full mb-0 p-0 gap-1.5 compact:w-full ${packClass("Plate", "frame")}`} role="status" label={props.title}>
      <Text class={`font-display text-xs font-semibold leading-tight tracking-label uppercase text-gold whitespace-nowrap m-0 overflow-hidden`}>
        {props.title}
      </Text>
      <View class="p-0 m-0 gap-1.5">
        <For each={props.objectives}>
          {(o, i) => (
          <View class={`group/row transition duration-317 opacity-72 gap-1 rounded-control hover:bg-surface-hover active:bg-surface-hover data-selected:bg-rim-soft data-selected:bg-none data-selected:shadow-[inset_3px_0px_0px_#e8e8ed] disabled:bg-transparent disabled:bg-none disabled:opacity-65 data-done:opacity-55 data-current:opacity-100 compact:hidden compact:data-current:flex compact:data-closing:flex`} data-closing={i() === props.objectives.length - 1 && o().done} data-current={!o().done && (i() === 0 || props.objectives[i() - 1]['done'])} data-done={o().done}>
            <View class="flex-row items-baseline gap-2">
              <View class="rotate-45 shadow-[inset_0px_0px_0px_2.288px_#ffcf5a] mx-0.5 size-2 group-data-current/row:shadow-[0px_0px_11.424px_#ffcf5acc] group-data-current/row:bg-none group-data-current/row:bg-gold group-data-done/row:bg-none group-data-done/row:bg-gold" />
              <Text class="text-reading font-semibold text-stroke-3 text-stroke-bg text-shadow-xs text-shadow-bg/55 m-0 group-data-done/row:line-through shrink">
                {o().label}
              </Text>
            </View>
            <Show when={!o().done}>
              <View class={`bg-track shadow-sunk h-1.25 ml-5 rounded-control overflow-hidden`}>
                <View class={`bg-gold h-full`} style={{ width: `${(100 * (o().progress))}%` }} />
                <Show when={o().steps > 1}>
                  <View class="absolute inset-0" />
                </Show>
              </View>
            </Show>
          </View>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ObjectiveTrackerProps>> = {
  default: {"title":"The Lighthouse","objectives":[{"id":"a","label":"Find the keeper","progress":1,"done":true,"steps":0},{"id":"b","label":"Light the lamps 2/5","progress":0.4,"done":false,"steps":5}]},
};
