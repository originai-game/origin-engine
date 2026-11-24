import type { Texture } from "@origingame/engine";
import { For, Image, packClass, Show, t, Text, View } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgTurnOrderProps {
  /** The round, on the plate under the portraits. */
  round: number;
  /** Keyed rows of { id, name, portrait, side, acting, down }, in the order they act: side ally, enemy or neutral; acting the one whose turn it is; portrait a texture, 0 none. */
  turns?: ReadonlyArray<{ id: string; name: string; portrait: Texture | 0; side: string; acting: boolean; down: boolean }>;
}

/**
 * The initiative: round portraits in acting order, the one acting first and
 * ringed, between two end flourishes, and a round plate on a rule under
 * them. All portraits share the width, so five fit a phone. Reports
 * nothing.
 */
export function RpgTurnOrder(props: RpgTurnOrderProps) {
  return (
    <View class={`flex-col items-center gap-1 w-full text-fg`} label={t("rpg.turns.name")}>
      <View class="flex-row items-center gap-1 w-full">
        <View class="shrink-0 w-6 h-8" />
        <View class={`grow grid grid-flow-col auto-cols-fr py-1 px-0 min-w-0`} role="list">
          <For each={props.turns ?? []} key={(u) => u.id}>
            {(u) => (
              <View class={`flex-col items-center p-1 min-w-0`} role="listitem" label={u().name} data-acting={u().acting} data-down={u().down} current={u().acting}>
                <View class={`relative w-full max-w-24 aspect-square overflow-hidden p-0.5 rounded-full bg-linear-to-b from-rim-strong via-rim via-45% to-rim-soft shadow-plate`}>
                  <Show when={u().portrait !== 0}>
                    <Image class={`size-full rounded-full bg-plate-hi`} src={u().portrait} />
                  </Show>
                </View>
              </View>
            )}
          </For>
        </View>
        <View class="shrink-0 w-6 h-8" />
      </View>
      <View class="flex-row items-center w-full gap-2">
        <View class={`grow h-px bg-rim`} role="separator" />
        <View class={`shrink py-1 px-6 text-lg font-bold tabular-nums whitespace-nowrap text-gold ${packClass("Chip", "frame")}`}>
          <Text class={`text-stroke-3 text-stroke-bg`}>{t("rpg.turns.round", { round: props.round })}</Text>
        </View>
        <View class={`grow h-px bg-rim`} role="separator" />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgTurnOrderProps>> = {
  default: {"round":3,"turns":[{"id":"a","name":"Aerin","portrait":0,"side":"ally","acting":true,"down":false},{"id":"o","name":"Oarsman","portrait":0,"side":"enemy","acting":false,"down":false}]},
};
