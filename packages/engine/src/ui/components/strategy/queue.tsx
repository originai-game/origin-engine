import type { Texture } from "@origingame/engine";
import { For, Icon, Image, packClass, Show, t, Text, View, type UiEvent } from "@origingame/engine/ui";
import "./strategy.en.ogt";

export interface StrategyQueueProps {
  /** What produces, as the game words it (Barracks queue); empty takes the kit's (Production). */
  title: string;
  /** The queue in the game's order, first in production, keyed rows of { id, name, picture, icon, count, progress, timeText, state, reason }: name for a reader; picture a texture (og.load.texture), 0 draws icon from the atlas; count a stack of the same order (×2, shown above 1); progress 0 to 1 done, a dark sweep over the first order's art for what is still to run; timeText its time left as the game words it (0:08), on the first order's tile, empty none; state building, waiting, paused or blocked (data-state; paused and blocked are also words after the tiles); reason why it is blocked (Need 3 supply), empty none. */
  items?: ReadonlyArray<{ id: string; name: string; picture: Texture | 0; icon: string; count: number; progress: number; timeText: string; state: string; reason: string }>;
  /** Orders queued, as the game counts them (stacks included), shown over limit on a tag. */
  queued: number;
  /** The most the queue holds: its free places are drawn as empty tiles after the orders; 0 hides the count and the free places. */
  limit: number;
  /** The game says no more can be queued: the Queue full word on the count's tag and data-full on the frame. */
  full: boolean;
  /** Cancel the order (a slot's key). */
  onCancel?: (e: UiEvent, id: string) => void;
}

/**
 * A production queue as one row: what produces in small capitals, the
 * order in production as a larger tile with a dark sweep for what is still
 * to run and its time left, the orders waiting as smaller tiles (a stack's
 * count on a tag), the free places as empty tiles, the count over the
 * limit, and Paused, Blocked with the reason, or Queue full as words.
 * Reports cancel (a tile's key); the game owns costs, times, refunds and
 * the limit.
 */
export function StrategyQueue(props: StrategyQueueProps) {
  return (
    <View class={`flex-row flex-wrap items-center gap-x-[0.714rem] gap-y-1 w-full min-w-0 text-fg compact:gap-x-[0.429rem]`} label={props.title !== "" ? props.title : t("strategy.queue.title")} data-full={props.full}>
      <Text class={`min-w-0 text-xs font-semibold tracking-[0.14em] uppercase whitespace-nowrap overflow-hidden text-fg/78 compact:text-xs compact:tracking-normal`}>
        {props.title !== "" ? props.title : t("strategy.queue.title")}
      </Text>
      <View class={`flex-row items-center gap-x-[0.429rem] min-w-0 shrink overflow-hidden compact:gap-x-1`} role="group" label={t("strategy.queue.items")}>
        <For each={props.items ?? []} key={(q) => q.id}>
          {(q, i) => (
          <View
            class={`shrink size-[1.914rem] compact:size-[1.783rem] p-[0.143rem] nine-chip-cut bg-white/75 data-state-blocked:bg-danger data-first:size-[2.514rem] compact:data-first:size-[2.286rem] ${packClass("Button", "plate")}`}
            role="button"
            label={t("strategy.queue.cancel", { name: q().name })}
            description={q().reason !== "" ? q().reason : q().timeText}
            data-first={i() === 0}
            data-state-blocked={q().state === "blocked"}
            data-state-paused={q().state === "paused"}
            onPress={(e) => props.onCancel?.(e, q().id)}
          >
            <View class="relative size-full overflow-hidden">
              <Show when={q().picture} fallback={
                  <View class="absolute inset-0 flex-row items-center justify-center bg-linear-to-b from-plate-hi to-plate-lo">
                    <Icon class={`size-[60%] text-fg`} name={q().icon} />
                  </View>
              }>
                <Image class="absolute inset-0 size-full" src={q().picture} />
              </Show>
              <Show when={q().state === "blocked"}>
                <View class="absolute inset-0 bg-black/55" />
              </Show>
              <Show when={q().state !== "waiting"}>
                <View
                  class="absolute inset-0 nine-tile bg-black/68"
                  wipe={1 - Math.min(1, Math.max(0, q().progress))}
                  wipeStart={Math.min(1, Math.max(0, q().progress)) * 2 * Math.PI}
                />
              </Show>
              <Show when={q().state !== "waiting" && q().timeText !== ""}>
                <View class="absolute inset-x-0 inset-y-0 flex-row items-center justify-center">
                  <Text class={`shrink text-[1.714rem] leading-none font-bold tabular-nums whitespace-nowrap text-stroke-3 text-stroke-bg`}>
                    {q().timeText}
                  </Text>
                </View>
              </Show>
              <Show when={q().count > 1}>
                <Text class={`absolute right-0 top-0 py-0 px-[0.143rem] leading-[1.2] text-xs font-bold whitespace-nowrap bg-[#060910e0] ${packClass("Chip", "frame")}`}>
                  {t("strategy.queue.stack", { count: q().count })}
                </Text>
              </Show>
            </View>
          </View>
          )}
        </For>
        <Show when={props.limit > props.queued}>
          <For each={Array.from({ length: Math.min(12, props.limit - props.queued) }, (_, i) => i)} key={(i) => i}>
            {() => (
              <View class={`shrink size-[1.914rem] compact:size-[1.783rem] nine-chip-cut bg-sheen`} />
            )}
          </For>
        </Show>
      </View>
      <Show when={props.limit > 0} fallback={
          <Show when={props.full}>
            <Text class={`shrink py-0 px-[0.429rem] leading-[1.6] text-xs font-bold whitespace-nowrap text-gold nine-chip-cut bg-[#060910e0] ${packClass("Chip", "frame")}`} label={t("strategy.queue.capacityName", { count: props.queued, limit: props.limit })}>
              {t("strategy.queue.full")}
            </Text>
          </Show>
      }>
        <Text class={`shrink py-0 px-[0.429rem] leading-[1.6] text-xs font-bold tabular-nums whitespace-nowrap text-fg data-full:text-gold nine-chip-cut bg-[#060910e0] ${packClass("Chip", "frame")}`} label={t("strategy.queue.capacityName", { count: props.queued, limit: props.limit })} data-full={props.full}>
          {props.full ? t("strategy.queue.fullCount", { count: props.queued, limit: props.limit }) : t("strategy.queue.capacity", { count: props.queued, limit: props.limit })}
        </Text>
      </Show>
      <For each={props.items ?? []} key={(q) => q.id}>
        {(q) => (
        <Show when={q().state === "paused" || q().state === "blocked"}>
          <Text class={`shrink py-0 px-[0.429rem] leading-[1.6] text-xs font-bold whitespace-nowrap text-fg data-state-paused:text-gold data-state-blocked:text-danger nine-chip-cut bg-[#060910e0] ${packClass("Chip", "frame")}`} role="status" data-state-paused={q().state === "paused"} data-state-blocked={q().state === "blocked"}>
            {q().state === "paused" ? t("strategy.queue.paused") : t("strategy.queue.blocked")}
          </Text>
          <Show when={q().reason !== ""}>
            <Text class={`shrink min-w-0 text-caption font-semibold text-danger whitespace-nowrap overflow-hidden`}>
              {q().reason}
            </Text>
          </Show>
        </Show>
        )}
      </For>
      <Show when={!(props.items ?? []).length}>
        <Text class={`shrink min-w-0 text-caption text-fg-muted whitespace-nowrap overflow-hidden`} role="status">
          {t("strategy.queue.empty")}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyQueueProps>> = {
  default: {"title":"Barracks queue","queued":3,"limit":5,"full":false,"items":[{"id":"o1","name":"Man-at-Arms","picture":0,"icon":"sword","count":1,"progress":0.64,"timeText":"0:08","state":"building","reason":""},{"id":"o2","name":"Archer","picture":0,"icon":"bolt","count":1,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Archer","picture":0,"icon":"bolt","count":1,"progress":0,"timeText":"","state":"waiting","reason":""}]},
  blocked: {"title":"","queued":4,"limit":5,"full":false,"items":[{"id":"o1","name":"Ballista","picture":0,"icon":"bomb","count":1,"progress":0,"timeText":"","state":"blocked","reason":"Need 3 supply"},{"id":"o2","name":"Archer","picture":0,"icon":"bolt","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Spearman","picture":0,"icon":"sword","count":1,"progress":0,"timeText":"","state":"waiting","reason":""}]},
  empty: {"title":"","queued":0,"limit":5,"full":false,"items":[]},
  full: {"title":"","queued":5,"limit":5,"full":true,"items":[{"id":"o1","name":"Knight","picture":0,"icon":"shield","count":1,"progress":0.18,"timeText":"0:33","state":"building","reason":""},{"id":"o2","name":"Knight","picture":0,"icon":"shield","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Healer","picture":0,"icon":"potion","count":2,"progress":0,"timeText":"","state":"waiting","reason":""}]},
  paused: {"title":"Barracks","queued":2,"limit":5,"full":false,"items":[{"id":"o1","name":"Pikeman","picture":0,"icon":"sword","count":1,"progress":0.4,"timeText":"0:11","state":"paused","reason":""},{"id":"o2","name":"Pikeman","picture":0,"icon":"sword","count":1,"progress":0,"timeText":"","state":"waiting","reason":""}]},
  overflow: {"title":"Royal Academy of Siegecraft and Applied Alchemy","queued":12,"limit":12,"full":true,"items":[{"id":"o1","name":"Trebuchet of the Burning Dawn, Masterwork Edition","picture":0,"icon":"bomb","count":1,"progress":0.91,"timeText":"10:00:00","state":"blocked","reason":"Requires the Engineers Guild and a free building plot near the wall"},{"id":"o2","name":"Ram","picture":0,"icon":"bomb","count":3,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o3","name":"Siege Tower","picture":0,"icon":"home","count":1,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o4","name":"Alchemist","picture":0,"icon":"potion","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o5","name":"Mortar","picture":0,"icon":"fire","count":1,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o6","name":"Sapper","picture":0,"icon":"gear","count":2,"progress":0,"timeText":"","state":"waiting","reason":""},{"id":"o7","name":"Bombard","picture":0,"icon":"bomb","count":2,"progress":0,"timeText":"","state":"waiting","reason":""}]},
};
