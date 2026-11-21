import { For, Scroll, Show, Text, type UiEvent, View } from "@origingame/engine/ui";
import { Button } from "../controls/button";

export interface DataListProps {
  /** The rows, a keyed collection of { id, title, detail, value }. */
  rows?: ReadonlyArray<{ id: string; title: string; detail: string; value: string }>;
  /** Rows are on their way: a busy line instead of the empty one. */
  loading?: boolean;
  /** What an empty list says. */
  empty: string;
  /** What a loading list says. */
  busy: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
}

/**
 * A HUD plate of rows: a name, its detail and its figure.
 */
export function DataList(props: DataListProps) {
  return (
    <View class={`rounded-tl-[13.712px] rounded-tr-none rounded-br-[13.712px] rounded-bl-none bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] text-fg w-full p-0`} busy={props.loading}>
      <Scroll class="grow shrink basis-0 w-full">
        <Show when={(props.rows ?? []).length} fallback={
            <Text class={`text-center text-fg/56 m-6`}>{props.loading ? props.busy : props.empty}</Text>
        }>
          <For each={props.rows ?? []}>
            {(r) => (
              <Button class={`w-full min-w-0 justify-stretch flex-row items-center gap-3 py-2 px-4 border-b border-rim-soft`} onPress={(e) => props.onChoose?.(e, r().id)}>
                <View class="flex-col items-start gap-0.5 grow shrink basis-0 min-w-0 text-start">
                  <Text class={`text-fg font-semibold text-start m-0`}>{r().title}</Text>
                  <Text class={`text-fg/60 text-start m-0 shrink`}>{r().detail}</Text>
                </View>
                <Text class={`text-fg text-[0.929rem] font-bold tabular-nums whitespace-nowrap shrink`}>{r().value}</Text>
              </Button>
            )}
          </For>
        </Show>
      </Scroll>
    </View>
  );
}

export const fixtures: Record<string, Partial<DataListProps>> = {
  default: {"loading":false,"empty":"Nothing here yet","busy":"Loading…","rows":[{"id":"a","title":"Copper ore","detail":"Mined in the north pit","value":"×12"},{"id":"b","title":"Silk","detail":"Bought in Harrow","value":"×3"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Open"} as const;
