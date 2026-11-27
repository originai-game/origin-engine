import { For, formatNumber, Show, t, Text, View } from "@origingame/engine/ui";
import "./rpg.en.strings.ts";

export interface RpgHitsProps {
  /** The latest damage and healing figures, newest first, keyed rows of { id, amount, kind, call, word }: kind normal, critical (gold and slanted, the largest), burn (damage over time in an ember ink, its word beside it), heal (green with a plus) or miss (on data-kind); call the words over a critical as the game words them (Critical · Weak point), empty for none; word the figure's kind as the game words it (burn), empty for none. The newest stands largest; when there are older ones it keeps room before it, where they step down and away (a phone shows the newest alone); the game trims the list and places it. */
  hits?: ReadonlyArray<{ id: string; amount: number; kind: string; call: string; word: string }>;
}

/**
 * Damage and healing figures beside the fight: a critical hit's figure
 * largest, gold and slanted under its call, older figures stepping down
 * and away, a burn in an ember ink with its word, a heal in green. The
 * game owns every figure and where the page puts them; read only.
 */
export function RpgHits(props: RpgHitsProps) {
  return (
    <Show when={(props.hits ?? []).length > 0}>
      <View class={`flex-col items-start max-w-full pointer-events-none text-stroke-3 text-stroke-bg text-shadow-tag p-2`} role="log" label={t("rpg.hits.name")}>
        <For each={props.hits ?? []} key={(h) => h.id}>
          {(h, i) => (
          <View class={`group/hits flex-col items-start max-w-full min-w-0 animate-pop transition duration-fast exit:opacity-0 data-later:mt-8 data-later:opacity-soft data-third:ml-55 data-third:mt-0 data-third:opacity-quiet data-lead:ml-37.5 compact:data-lead:ml-0 compact:data-later:hidden`}
            data-lead={i() === 0 && (props.hits ?? []).length > 1}
            data-later={i() >= 1}
            data-third={i() >= 2}
            data-normal={h().kind === "normal"}
            data-burn={h().kind === "burn"}
            data-critical={h().kind === "critical"}
            data-heal={h().kind === "heal"}
            data-miss={h().kind === "miss"}>
            <Show when={h().call}>
              <Text class={`max-w-full min-w-0 text-sm font-semibold tracking-label uppercase text-gold whitespace-normal compact:text-xs compact:tracking-normal`}>
                {h().call}
              </Text>
            </Show>
            <View class="flex-row items-baseline min-w-0 max-w-full gap-1">
              <Show when={h().kind !== "miss"}>
                <Text class={`shrink text-display-md leading-none font-bold tabular-nums whitespace-nowrap group-data-critical/hits:text-display-xl group-data-critical/hits:italic group-data-critical/hits:text-gold group-data-burn/hits:text-legendary group-data-heal/hits:text-success compact:text-title compact:group-data-critical/hits:text-4xl`}>
                  {h().kind === "heal" ? t("rpg.hits.heal", { amount: formatNumber(h().amount) }) : formatNumber(h().amount)}
                </Text>
              </Show>
              <Show when={h().word}>
                <Text class={`min-w-0 text-sm font-bold whitespace-nowrap overflow-hidden group-data-burn/hits:text-legendary group-data-miss/hits:text-title group-data-miss/hits:opacity-quiet`}>
                  {h().word}
                </Text>
              </Show>
            </View>
          </View>
          )}
        </For>
      </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<RpgHitsProps>> = {
  default: {"hits":[{"id":"h3","amount":2481,"kind":"critical","call":"Critical · Weak point","word":""},{"id":"h2","amount":642,"kind":"normal","call":"","word":""},{"id":"h1","amount":118,"kind":"burn","call":"","word":"burn"}]},
  heal: {"hits":[{"id":"r1","amount":320,"kind":"heal","call":"","word":""}]},
  miss: {"hits":[{"id":"m1","amount":0,"kind":"miss","call":"","word":"Miss"}]},
  overflow: {"hits":[{"id":"o1","amount":12345678,"kind":"critical","call":"Critical · Weak point · Overkill · Backstab","word":""},{"id":"o2","amount":999999,"kind":"burn","call":"","word":"searing flame"}]},
};
