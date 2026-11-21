import { For, Scroll, Text, type UiEvent, View } from "@origingame/engine/ui";

export interface TableProps {
  /** The headings, a keyed collection of { id, label }. */
  columns: ReadonlyArray<{ id: string; label: string }>;
  /** The body's cells row by row, a keyed collection of { id, row, text }: three to a row (eject it to change the columns). A press sends choose with the cell's id; its row field names the row. */
  cells: ReadonlyArray<{ id: string; row: string; text: string }>;
  /** The table's accessible name. */
  label: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * A HUD plate of plain rows: the first column a name, the others figures.
 */
export function Table(props: TableProps) {
  return (
    <View class={`rounded-tl-[13.712px] rounded-tr-none rounded-br-[13.712px] rounded-bl-none bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] text-fg w-full p-0`} role="table" label={props.label}>
      <View class={`min-h-12 border-b border-material-edge grid grid-cols-[2fr_1fr_1fr] w-full p-2`} role="row">
        <For each={props.columns}>
          {(c, i) => (
          <Text class={`tracking-[0.14em] uppercase text-fg/50 text-[0.857rem] font-semibold px-2 m-0 data-end:text-end`} role="columnheader" data-end={i() > 0}>
            {c().label}
          </Text>
          )}
        </For>
      </View>
      <Scroll class="grow shrink basis-0">
        <View class="grid grid-cols-[2fr_1fr_1fr] w-full px-2">
          <For each={props.cells}>
            {(c, i) => (
            <View class={`w-full min-h-11 text-start border-b border-material-edge px-2 data-figure:justify-end data-figure:text-end data-figure:font-bold`} role="cell" onPress={(e) => props.onChoose?.(e, c().id)} data-figure={i() % 3 !== 0}>
              <Text class="whitespace-normal data-figure:tabular-nums">{c().text}</Text>
            </View>
            )}
          </For>
        </View>
      </Scroll>
    </View>
  );
}

export const fixtures: Record<string, Partial<TableProps>> = {
  default: {"label":"Standings","columns":[{"id":"n","label":"Team"},{"id":"w","label":"Won"},{"id":"p","label":"Points"}],"cells":[{"id":"r.n","row":"r","text":"Ravens"},{"id":"r.w","row":"r","text":"9"},{"id":"r.p","row":"r","text":"27"},{"id":"g.n","row":"g","text":"Gulls"},{"id":"g.w","row":"g","text":"7"},{"id":"g.p","row":"g","text":"21"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Open row"} as const;
