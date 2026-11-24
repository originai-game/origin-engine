import { For, Icon, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import { Button } from "../../controls/button";
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
 * Reports choose (a row's key) and retry.
 *
 * Dropped from the .svelte: the plate's rolled corner rims (the cut shape
 * carries no border here), and the rows' hairline separators (each row is
 * its own plate). The chosen row's wash is one flat colour.
 */
export function DataGrid(props: DataGridProps) {
  return (
    <View class={`w-full p-0 gap-0 ${packClass("Panel", "frame")}`} role="grid" label={props.label} busy={props.loading}>
      <View class={`min-h-8 border-b border-border-muted grid gap-x-3 w-full py-2 px-3 compact:flex compact:gap-x-3 compact:flex-wrap compact:gap-y-0.5 compact:items-baseline`} role="row" style={{ gridTemplateColumns: String(props.tracks ||"1fr 1fr 1fr 1fr") }}>
        <For each={props.columns} key={(c) => c.id}>
          {(c, i) => (
          <Text class={`font-semibold text-xs leading-4.5 text-fg-muted whitespace-nowrap tracking-label uppercase data-numeric:text-end compact:grow-0 compact:shrink-0 compact:basis-auto compact:w-auto compact:order-3 compact:data-first:whitespace-normal compact:data-first:grow compact:data-first:shrink compact:data-first:basis-0 compact:data-first:order-0 compact:data-first:min-w-0 compact:data-last:order-1 compact:data-middle:hidden`} role="columnheader" data-numeric={c().numeric} data-first={i() === 0} data-last={i() === props.columns.length - 1} data-middle={i() > 0 && i() < props.columns.length - 1}>
            {c().label}
          </Text>
          )}
        </For>
      </View>
      <Show when={props.loading} fallback={
          <Show when={props.error} fallback={
              <Show when={!props.rows.length} fallback={
                  <View class="w-full h-60 overflow-y-auto">
                    <For each={props.rows} key={(r) => r.id}>
                      {(r) => (
                      <View class={`group/row relative w-full min-w-0 items-baseline grid gap-x-3 py-1 px-3 compact:flex compact:flex-wrap compact:gap-x-3 compact:gap-y-0.5 compact:py-2 hover:bg-surface-hover ${r().id === props.selected ? packClass("Row", "frame", "selected") : packClass("Row", "frame")} data-down:opacity-dim`} role="row" selected={r().id === props.selected} onPress={(e) => props.onChoose?.(e, r().id)} data-selected={r().id === props.selected} style={{ gridTemplateColumns: String(props.tracks ||"1fr 1fr 1fr 1fr") }}>
                        <View class="absolute left-0 inset-y-0 w-0.75 bg-hud-accent hidden group-data-selected/row:block" />
                        <For each={r().cells} key={(c) => c.id}>
                          {(c, i) => (
                          <>
                            <Show when={i() === 1}>
                              <View class="hidden compact:block compact:order-2 compact:basis-full h-0" />
                            </Show>
                            <Text class={`min-w-0 text-start text-fg-muted whitespace-nowrap text-ellipsis overflow-hidden data-first:text-fg data-first:font-semibold data-numeric:text-end data-numeric:whitespace-nowrap data-numeric:text-clip data-numeric:overflow-visible data-numeric:text-fg data-numeric:font-bold data-numeric:tabular-nums compact:order-3 compact:shrink-0 compact:basis-auto compact:w-auto compact:data-first:order-0 compact:data-first:grow compact:data-first:shrink compact:data-first:basis-0 compact:data-first:min-w-0 compact:data-first:whitespace-normal compact:data-last:order-1 compact:data-middle:text-caption`} role="gridcell" data-numeric={c().numeric} data-first={i() === 0} data-last={i() === r().cells.length - 1} data-middle={i() > 0 && i() < r().cells.length - 1}>
                              {c().text}
                            </Text>
                          </>
                          )}
                        </For>
                      </View>
                      )}
                    </For>
                  </View>
              }>
                <View class={`w-full items-center p-6 gap-3`} role="status">
                  <View class={`grow shrink basis-0 bg-well inset-ring-hairline inset-ring-border justify-center items-center gap-3 size-16 rounded-capsule`}>
                    <Icon class="size-4" name="bag" />
                  </View>
                  <Text class={`font-semibold text-lg text-fg text-center w-full m-0`}>
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
              <View class={`grow shrink basis-0 bg-danger inset-ring-regular inset-ring-danger justify-center items-center gap-3 size-16 rounded-capsule`}>
                <Icon class="text-danger" name="close" />
              </View>
              <Text class={`font-semibold text-lg text-fg text-center w-full m-0`}>
                {props.error}
              </Text>
              <Button variant="primary" onPress={(e) => props.onRetry?.(e)}>
                <Text class="whitespace-normal">{t("component.data-grid.retry")}</Text>
              </Button>
            </View>
          </Show>
      }>
        <View class="p-0 gap-0" role="status" label={t("component.data-grid.loading")}>
          <View class="w-full border-b border-border-muted py-2 px-3">
            <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
              <View class={`absolute left-0 w-2/5 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </View>
          <View class="w-full border-b border-border-muted py-2 px-3">
            <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
              <View class={`absolute left-0 w-2/5 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </View>
          <View class="w-full border-b border-border-muted py-2 px-3">
            <View class={`bg-bg-subtle w-full h-3 rounded-capsule overflow-hidden`}>
              <View class={`absolute left-0 w-2/5 pointer-events-none bg-linear-to-r from-transparent via-border to-transparent inset-y-0 animate-shimmer`} />
            </View>
          </View>
          <View class="w-full border-b border-border-muted py-2 px-3">
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
  loading: {"label":"Inventory","tracks":"","selected":"","loading":true,"error":"","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[]},
  error: {"label":"Inventory","tracks":"","selected":"","loading":false,"error":"The vault did not answer.","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[]},
  empty: {"label":"Inventory","tracks":"","selected":"","loading":false,"error":"","empty":"The satchel holds nothing.","hint":"Fill it at the merchant.","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[]},
  selected: {"label":"Inventory","tracks":"","selected":"a","loading":false,"error":"","columns":[{"id":"name","label":"Item","numeric":false},{"id":"kind","label":"Type","numeric":false},{"id":"qty","label":"Qty","numeric":true},{"id":"value","label":"Value","numeric":true}],"rows":[{"id":"a","cells":[{"id":"name","text":"Ember Blade","numeric":false},{"id":"kind","text":"Weapon","numeric":false},{"id":"qty","text":"1","numeric":true},{"id":"value","text":"1,250","numeric":true}]},{"id":"b","cells":[{"id":"name","text":"Health Potion","numeric":false},{"id":"kind","text":"Consumable","numeric":false},{"id":"qty","text":"12","numeric":true},{"id":"value","text":"40","numeric":true}]}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Select","retry":"Retry"} as const;
