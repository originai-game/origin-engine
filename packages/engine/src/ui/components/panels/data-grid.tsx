import { For, Icon, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./data-grid.en.ogt";

export interface DataGridProps {
  /** The headings, a keyed collection of { id, label, numeric }: numeric columns align to the end in tabular figures. */
  columns: ReadonlyArray<{ id: string; label: string; numeric: boolean }>;
  /** The columns' widths as CSS grid tracks, one per column; empty shares the width equally between four. A numeric column's track holds its widest figure in text-scaled units (calc(96 * var(--font-step-px))), so it grows with the player's text size; a figure is never cut, and the text columns (fr) give way. */
  tracks: string;
  /** The rows, a keyed collection of { id, cells }: cells a keyed list of { id, text, numeric }, in the columns' order. Only the rows in view are built. */
  rows: ReadonlyArray<{ id: string; cells: ReadonlyArray<{ id: string; text: string; numeric: boolean }> }>;
  /** The id of the selected row; empty is none. */
  selected: string;
  /** What a screen reader calls the grid. */
  label: string;
  /** Rows are on their way: busy placeholder rows under the header. */
  loading?: boolean;
  /** An error to show with Retry instead of the rows; empty is none. */
  error: string;
  /** What an empty grid says; empty is Nothing here yet. */
  empty: string;
  /** A line under the empty message. */
  hint: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The retry event, with the row's key where it comes from a row. */
  onRetry?: (e: UiEvent, key?: string) => void;
}

/**
 * The HUD's smoked-glass plate: headings in small tracked capitals over
 * a faint rule, rows dense and plain so the data reads first, figures
 * condensed and tabular. The chosen row is washed in the accent from its
 * leading edge. In a narrow shell each row becomes a two-line card: the
 * first column and the last on one line, the rest as a caption under.
 */
export function DataGrid(props: DataGridProps) {
  return (
    <View class={`rounded-tl-[13.712px] rounded-tr-none rounded-br-[13.712px] rounded-bl-none bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] text-accent-hover w-full p-0 gap-0`} role="grid" label={props.label} busy={props.loading}>
      <View class={`min-h-8 border-b border-material-edge grid gap-x-3 w-full py-2 px-3 compact:flex compact:gap-x-3 compact:flex-wrap compact:gap-y-0.5 compact:items-baseline`} role="row" style={{ gridTemplateColumns: String(props.tracks || "1fr 1fr 1fr 1fr") }}>
        <For each={props.columns}>
          {(c, i) => (
          <Text class={`font-semibold text-[0.857rem] leading-4.5 text-accent-hover/50 whitespace-nowrap tracking-[0.14em] uppercase data-numeric:text-end compact:grow-0 compact:shrink-0 compact:basis-auto compact:w-auto compact:order-3 compact:data-first:whitespace-normal compact:data-first:grow compact:data-first:shrink compact:data-first:basis-0 compact:data-first:order-0 compact:data-first:min-w-0 compact:data-last:order-1 compact:data-middle:hidden`} role="columnheader" data-numeric={c().numeric} data-first={i() === 0} data-last={i() === props.columns.length - 1} data-middle={i() > 0 && i() < props.columns.length - 1}>
            {c().label}
          </Text>
          )}
        </For>
      </View>
      <Show when={props.loading} fallback={
          <Show when={props.error} fallback={
              <Show when={!props.rows.length} fallback={
                  {/* TODO(world UI): <list id="Rows"> is not ported (bb11's later head) */}
              }>
                <View class={`w-full items-center p-6 gap-3`} role="status">
                  <View class={`grow shrink basis-0 bg-[#04060c9e] shadow-[inset_0px_0px_0px_1px_#ffffff1a] justify-center items-center gap-3 size-16 rounded-capsule`}>
                    <Icon name="bag" />
                  </View>
                  <Text class={`font-semibold text-lg text-accent-hover text-center w-full m-0`}>
                    {props.empty || t("component.data-grid.empty")}
                  </Text>
                  <Show when={props.hint}>
                    <Text class={`text-caption text-fg-muted text-center w-full m-0`}>
                      {props.hint}
                    </Text>
                  </Show>
                </View>
              </Show>
          }>
            <View class={`w-full items-center p-6 gap-3`} role="alert" label={props.error}>
              <View class={`grow shrink basis-0 bg-danger/6 shadow-[inset_0px_0px_0px_1.5px_#ff9599] justify-center items-center gap-3 size-16 rounded-capsule`}>
                <Icon class="text-danger" name="close" />
              </View>
              <Text class={`font-semibold text-lg text-accent-hover text-center w-full m-0`}>
                {props.error}
              </Text>
              <View class={`text-accent min-h-11 max-w-full`} role="button" onPress={(e) => props.onRetry?.(e)}>
                <Text class="whitespace-normal">{t("component.data-grid.retry")}</Text>
              </View>
            </View>
          </Show>
      }>
        <View class="p-0 gap-0" role="status" label={t("component.data-grid.loading")}>
          <View class="w-full border-b border-material-edge py-2 px-3">
            <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
              <View class={`absolute left-0 w-2/5 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </View>
          <View class="w-full border-b border-material-edge py-2 px-3">
            <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
              <View class={`absolute left-0 w-2/5 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </View>
          <View class="w-full border-b border-material-edge py-2 px-3">
            <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
              <View class={`absolute left-0 w-2/5 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </View>
          <View class="w-full border-b border-material-edge py-2 px-3">
            <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
              <View class={`absolute left-0 w-2/5 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<DataGridProps>> = {
  default: {"label":"Inventory","tracks":"2fr 1fr calc(48 * var(--font-step-px)) calc(96 * var(--font-step-px))","selected":"b","loading":false,"error":"","empty":"","hint":"","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[{"id":"a","cells":[{"id":"name","text":"Ember Blade","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,250","numeric":true}]},{"id":"b","cells":[{"id":"name","text":"Health Potion","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"40","numeric":true}]},{"id":"c","cells":[{"id":"name","text":"Iron Shield","numeric":false},{"id":"kind","text":"Armour","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"380","numeric":true}]},{"id":"d","cells":[{"id":"name","text":"Silver Key","numeric":false},{"id":"kind","text":"Quest","numeric":false},{"id":"qty","text":"2","numeric":true},{"id":"value","text":"0","numeric":true}]},{"id":"e","cells":[{"id":"name","text":"Moonstone","numeric":false},{"id":"kind","text":"Gem","numeric":false},{"id":"qty","text":"3","numeric":true},{"id":"value","text":"900","numeric":true}]}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Select","retry":"Retry"} as const;
