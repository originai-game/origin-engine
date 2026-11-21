import {For, Icon, Show, Text, packClass, type UiEvent, View} from "@origingame/engine/ui";
import { Button } from "../controls/button";

export interface PaginationProps {
  /** The page buttons, a keyed collection of { id, label, current }; a gap is { id, label: "…", gap: true }. */
  pages: ReadonlyArray<{ id: string; label: string; current?: boolean; gap?: boolean }>;
  /** The navigation's accessible name (Pages). */
  label: string;
  /** The first page is shown: Previous takes no press. */
  atStart?: boolean;
  /** The last page is shown: Next takes no press. */
  atEnd?: boolean;
  /** The next event, with the row's key where it comes from a row. */
  onNext?: (e: UiEvent, key?: string) => void;
  /** The page event, with the row's key where it comes from a row. */
  onPage?: (e: UiEvent, key?: string) => void;
  /** The previous event, with the row's key where it comes from a row. */
  onPrevious?: (e: UiEvent, key?: string) => void;
}

/**
 * A row of cut keys, the current page lit.
 */
export function Pagination(props: PaginationProps) {
  return (
    <View class={`flex-row items-center text-fg max-w-full p-0 gap-1 disabled:text-fg-subtle`} label={props.label}>
      <Button class={`min-w-11 min-h-11 p-0 rounded-[4.576px] ${packClass("Button", "plate")}`} disabled={props.atStart} onPress={(e) => props.onPrevious?.(e)} label={props.label}>
        <Icon class="size-4" name="chevronLeft" />
      </Button>
      <View class="flex-row flex-wrap items-center shrink justify-center p-0 m-0 gap-1">
        <For each={props.pages}>
          {(p) => (
          <View class="shrink">
            <Show when={p().gap}>
              <Text class={`text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/50 py-0 px-1`}>
                {p().label}
              </Text>
            </Show>
            <Show when={!p().gap}>
              <Button class={`bg-tile shadow-[inset_0px_0px_0px_1px_#ffffff1a] min-w-11 min-h-11 font-bold rounded-[4.576px] data-variant-primary:text-accent data-current:text-[#0b0f18] ${packClass("Button", "plate")}`} onPress={(e) => props.onPage?.(e, p().id)} data-current={p().current} data-variant-primary={(p().current ? "primary" : "quiet") === "primary"} data-variant-quiet={(p().current ? "primary" : "quiet") === "quiet"}>
                <Text class="tabular-nums">{p().label}</Text>
              </Button>
            </Show>
          </View>
          )}
        </For>
      </View>
      <Button class={`min-w-11 min-h-11 p-0 rounded-[4.576px] ${packClass("Button", "plate")}`} disabled={props.atEnd} onPress={(e) => props.onNext?.(e)} label={props.label}>
        <Icon class="size-4" name="chevronRight" />
      </Button>
    </View>
  );
}

export const fixtures: Record<string, Partial<PaginationProps>> = {
  default: {"label":"Pages","atStart":false,"atEnd":false,"pages":[{"id":"1","label":"1","current":false},{"id":"2","label":"2","current":true},{"id":"3","label":"3","current":false},{"id":"gap","label":"…","gap":true},{"id":"9","label":"9","current":false}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"previous":"Previous page","next":"Next page","page":"Go to page"} as const;
