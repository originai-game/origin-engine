import { For, Show, Text, View, t } from "@origingame/engine/ui";
import "./celebration.en.ogt";

export interface CelebrationProps {
  /** Which one: confetti (falling over its box from the top edge), burst (a ring and sparks thrown from its centre), levelUp (a ring of light, rays turning behind the title and the new level, sparks rising) or fireworks (shells bursting across its top). */
  kind: string;
  /** Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once. */
  shots?: ReadonlyArray<{ id: string }>;
  /** levelUp: the big word; empty is Level up. */
  title?: string;
  /** levelUp: the new level (12); empty is none. */
  value?: string;
}

/**
 * A reward over the HUD: a burst's ring, a level-up's ring of light with
 * its word in gold and its new level, or fireworks' shells. Each play
 * runs once per shot. The surfaces the old document used have no TSX
 * form: the particle emitters (confetti, sparks, rising embers), the
 * rays' polygon, the shells' burst keyframe and the play sounds are
 * dropped; the ring, halo, title and level stand in on the built-in
 * animate classes (ping, stamp, pop).
 */
export function Celebration(props: CelebrationProps) {
  const level = () => (props.kind ?? "") == "levelUp";
  return (
    <View class="absolute inset-0 text-fg text-shadow-2xs">
      <For each={props.shots ?? []} key={(s) => s.id}>
        {() => (
          <View class="absolute inset-0 flex-col items-center justify-center" data-level={level()} data-burst={(props.kind ?? "") == "burst"} data-fireworks={(props.kind ?? "") == "fireworks"}>
            <Show when={level()}>
              <View class="relative w-[30rem] max-w-full min-h-[20.571rem] flex-col items-center justify-center">
                <View class="absolute size-[18.857rem] rounded-full border-[0.286rem] border-gold shadow-[0px_0px_1.143rem_#ffd23f,inset_0px_0px_1.143rem_#ffd23f] bg-gold-fg/31 animate-stamp compact:size-[6.8rem]" />
                <Text class="relative max-w-full mb-3 text-[1.714rem] font-bold tracking-[0.14em] uppercase text-center text-gold animate-stamp">
                  {props.title || t("component.celebration.level-up")}
                </Text>
                <Show when={props.value}>
                  <Text class="relative text-[8.571rem] font-bold tabular-nums leading-flat whitespace-nowrap text-fg shadow-none animate-pop compact:text-[2.429rem]">{props.value}</Text>
                </Show>
              </View>
            </Show>
            <Show when={(props.kind ?? "") == "burst"}>
              <View class="size-16 rounded-full border-[0.286rem] border-white shadow-[0px_0px_0.571rem_#ffffff] animate-ping" />
            </Show>
            <Show when={(props.kind ?? "") == "fireworks"}>
              <View class="absolute top-0 inset-x-0 h-[17.143rem] flex-row justify-between py-0 px-12">
                <View class="size-2 rounded-full bg-gold animate-pop" />
                <View class="size-2 rounded-full bg-gold animate-pop" />
                <View class="size-2 rounded-full bg-gold animate-pop" />
                <View class="size-2 rounded-full bg-gold animate-pop" />
                <View class="size-2 rounded-full bg-gold animate-pop" />
              </View>
            </Show>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<CelebrationProps>> = {
  default: {"kind":"levelUp","shots":[{"id":"1"}],"title":"","value":"12"},
};
