import { For, Icon, packClass, Scroll, Text, View } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgStatsProps {
  /** The panel's heading (Stats). */
  title: string;
  /** The groups, keyed rows of { id, column }: column a or b; a rule parts one group from the next in a column. */
  groups?: ReadonlyArray<{ id: string; column: string }>;
  /** The figures, keyed rows of { id, group, icon, label, value }: group a group's id; value as the game words it. */
  rows?: ReadonlyArray<{ id: string; group: string; icon: string; label: string; value: string }>;
}

/**
 * A character's figures in two columns of grouped rows, each an icon, a
 * label and a value. Read only; fills its height and scrolls inside it.
 */
export function RpgStats(props: RpgStatsProps) {
  const column = (which: string) => (
    <View class={`flex-col min-w-0 grow basis-0`} role="table">
      <For each={props.groups ?? []} key={(g) => g.id}>
        {(g) => g().column === which ? (
          <View class={`flex-col w-full pb-2`} role="group">
            <For each={props.rows ?? []} key={(r) => r.id}>
              {(r) => r().group === g().id ? (
                <View class={`flex-row items-center min-h-8 py-1 w-full inset-shadow-underline inset-shadow-sheen`} role="row">
                  <Icon class={`shrink size-4 text-fg-muted`} name={r().icon} />
                  <Text class={`grow min-w-0 text-caption font-semibold text-fg-muted`} role="rowheader">{r().label}</Text>
                  <Text class={`shrink whitespace-nowrap text-lg font-bold tabular-nums text-fg`} role="cell">{r().value}</Text>
                </View>
              ) : null}
            </For>
          </View>
        ) : null}
      </For>
    </View>
  );
  return (
    <View class={`flex-col size-full p-4 compact:p-2 nine-panel-cut bg-raised-pressed text-fg ${packClass("Panel", "frame")}`} label={props.title}>
      <View class={`shrink flex-row items-center justify-center gap-3 w-full`}>
        <View class="shrink-0 w-12 h-4" />
        <Text class={`min-w-0 text-title leading-tight font-bold tracking-sm uppercase text-fg text-center whitespace-normal`} role="heading" level={2}>
          {props.title}
        </Text>
        <View class="shrink-0 w-12 h-4" />
      </View>
      <Scroll class={`grow min-h-0 w-full`}>
        <View class={`flex-row flex-wrap items-stretch gap-3 w-full px-1`}>
          {column("a")}
          <View class={`shrink-0 w-px self-stretch bg-lit compact:hidden`} />
          {column("b")}
        </View>
      </Scroll>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgStatsProps>> = {
  default: {"title":"Stats","groups":[{"id":"res","column":"a"},{"id":"rst","column":"b"}],"rows":[{"id":"hp","group":"res","icon":"heart","label":"Health","value":"2,850"},{"id":"fire","group":"rst","icon":"fire","label":"Fire resistance","value":"72%"}]},
};
