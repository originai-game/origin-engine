import { For, Text, View } from "@origingame/engine/ui";

export interface KeyValuesProps {
  /** The rows, a keyed collection of { id, key, value, delta } (keyValueRows; statCompare when delta is set: + better, - worse). */
  rows: ReadonlyArray<{ id: string; key: string; value: string; delta: string }>;
}

export function KeyValues(props: KeyValuesProps) {
  return (
    <View class={`text-fg w-full p-0 m-0 disabled:text-fg-subtle`}>
      <For each={props.rows}>
        {(r) => (
        <View class={`flex-row text-fg transition duration-317 items-center min-h-9 border-b border-rim py-1 px-0 gap-2 rounded-control hover:bg-surface-hover active:bg-surface-hover data-selected:bg-rim-soft data-selected:bg-none data-selected:text-fg data-selected:inset-shadow-bar data-selected:inset-shadow-accent disabled:bg-transparent disabled:bg-none disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`}>
          <Text class={`tracking-xs text-fg-muted grow shrink basis-0 m-0`}>
            {r().key}
          </Text>
          <Text class={`tabular-nums text-fg font-semibold m-0 shrink`}>
            {r().value}
          </Text>
          <Text class={`min-w-12 text-end tabular-nums text-fg-muted m-0 data-up:text-success data-down:text-danger shrink data-hidden:hidden`} data-hidden={!r().delta} data-up={r().delta[0] === "+"} data-down={r().delta[0] === "-" || r().delta[0] === "−"}>
            {r().delta}
          </Text>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<KeyValuesProps>> = {
  default: {"rows":[{"id":"atk","key":"Attack","value":"42","delta":"+6"},{"id":"def","key":"Defence","value":"18","delta":"-2"},{"id":"wt","key":"Weight","value":"3.5 kg","delta":""}]},
};
