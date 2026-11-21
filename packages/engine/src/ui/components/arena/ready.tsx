import { For, Scroll, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import "./ready.en.ogt";

export interface ReadyPlayer {
  id: string;
  /** What a reader calls them. */
  name: string;
  /** ready, declined, offline, or pending (empty). Order belongs to the game. */
  state?: string;
}

export interface ArenaReadyProps {
  /** Heading; empty uses Ready check. */
  title?: string;
  /** Match or reason line; empty hides it. */
  detail?: string;
  /** Game-owned time remaining; empty hides it. This component never starts or expires a timer. */
  timeText?: string;
  /** The players asked. */
  players?: ReadonlyArray<ReadyPlayer>;
  /** Local response: ready, declined or pending (empty). Both buttons remain available to change a response while canRespond. */
  response?: string;
  /** The game allows a local response. False disables both actions. Default true. */
  canRespond?: boolean;
  /** The check has ended; no more responses. timeText is hidden. */
  closed?: boolean;
  /** Waiting for participants; stale rows are hidden and responses disabled. */
  loading?: boolean;
  /** Accept. */
  onAccept?: (e: UiEvent) => void;
  /** Decline. */
  onDecline?: (e: UiEvent) => void;
}

/**
 * The ready check on a smoked cut card: the clock in gold, a row per
 * player with its answer as a word, its leading edge green once ready and
 * red when refused; Accept is green. The game owns readiness, timeout and
 * acceptance; the actions report from the stable component mount, no
 * local readiness is faked. Approximations: the card's cut corner is
 * rounded and the scroll fade at the list's foot is dropped.
 */
const CARD = "w-full max-h-full p-4 rounded-tl-[0.714rem] bg-linear-to-b from-[#232837ed] to-[#1a1e29f2] shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.14),inset_0px_1px_0px_0px_rgb(255_255_255/0.5)]";
const BUTTON = "flex-1 basis-44 flex-row items-center justify-center gap-x-1 min-h-11 px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-linear-to-b from-[#2c3140eb] to-[#1b1f29f2] shadow-[inset_0px_0px_0px_1px_rgb(255_255_255/0.24),inset_0px_1px_0px_0px_rgb(255_255_255/0.6)] rounded-tl-[0.429rem] hover:bg-none hover:bg-linear-to-b hover:from-[#373d4ff2] hover:to-[#232837f5] disabled:opacity-45 data-selected:shadow-[inset_0px_0px_0px_0.286rem_#38e8ff,inset_0px_0.429rem_0px_0px_rgb(255_255_255/0.5)]";
const STATE_EDGE = "shadow-[inset_3px_0px_0px_0px_rgb(255_255_255/0.3)]";

export function ArenaReady(props: ArenaReadyProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("ready.title");
  const canAct = () => (props.canRespond ?? true) && !props.closed && !props.loading && (props.players ?? []).length > 0;
  return (
    <View class={`flex-col gap-y-2 text-fg ${CARD}`} label={heading()} busy={props.loading}>
      <Text class="text-[1.714rem] leading-[1.1] font-bold uppercase tracking-[0.04em] text-fg text-center">{heading()}</Text>
      <Show when={(props.detail ?? "") != ""}>
        <Text class="text-[0.929rem] font-semibold text-white/66 text-center">{props.detail}</Text>
      </Show>
      <Show when={props.closed} fallback={
        <Show when={(props.timeText ?? "") != ""}>
          <Text class="text-[3.857rem] leading-none font-bold tabular-nums text-[#ffcf5a] text-center">{props.timeText}</Text>
        </Show>
      }>
        <Text class="text-[0.929rem] font-bold tracking-[0.14em] uppercase text-white/66 text-center" role="status">{t("ready.closed")}</Text>
      </Show>
      <Show when={props.loading} fallback={
        <Show when={(props.players ?? []).length}>
          <Scroll class="min-h-0">
            <View class="flex-col gap-y-1 pb-4" role="list" label={t("ready.players")}>
              <For each={props.players ?? []} key={(p) => p.id}>
                {(p) => (
                  <View
                    class={`group/readyrow flex-row flex-wrap items-center justify-between gap-x-2 min-h-8 py-1 px-3 bg-linear-to-r from-[#282e3cc7] to-[#1c202bd1] ${STATE_EDGE} data-ready:shadow-[inset_3px_0px_0px_0px_#55e39a] data-declined:shadow-[inset_3px_0px_0px_0px_#ff4d5e]`}
                    role="listitem"
                    data-ready={p().state == "ready"}
                    data-declined={p().state == "declined"}
                  >
                    <Text class={`flex-1 basis-16 min-w-0 text-[1.071rem] font-semibold text-fg data-offline:text-white/66`} data-offline={p().state == "offline"}>{p().name}</Text>
                    <Text
                      class={`ml-auto text-[0.929rem] leading-[1.3] font-bold tracking-[0.14em] uppercase text-white/66 text-center data-ready:text-[#55e39a] data-declined:text-[#ff7a83]`}
                      data-ready={p().state == "ready"}
                      data-declined={p().state == "declined"}
                    >
                      {p().state == "ready" ? t("ready.accepted") : p().state == "declined" ? t("ready.declined") : p().state == "offline" ? t("ready.offline") : t("ready.pending")}
                    </Text>
                  </View>
                )}
              </For>
            </View>
          </Scroll>
        </Show>
      }>
        <Text class="text-[0.929rem] font-semibold text-white/66 text-center" role="status">{t("ready.loading")}</Text>
      </Show>
      <Show when={!props.loading && (props.players ?? []).length == 0}>
        <Text class="text-[0.929rem] font-semibold text-white/66 text-center" role="status">{t("ready.empty")}</Text>
      </Show>
      <View class="flex-row flex-wrap gap-x-1 gap-y-1">
        <View
          class={`${BUTTON} text-[#011e0f] bg-none bg-[#55e39a] hover:bg-[#55e39a]`}
          role="button"
          label={t("ready.accept")}
          pressed={props.response == "ready"}
          disabled={!canAct()}
          onPress={(e) => props.onAccept?.(e)}
          data-selected={props.response == "ready"}
        >
          <Text class="text-[#011e0f]">{t("ready.accept")}</Text>
        </View>
        <View
          class={BUTTON}
          role="button"
          label={t("ready.decline")}
          pressed={props.response == "declined"}
          disabled={!canAct()}
          onPress={(e) => props.onDecline?.(e)}
          data-selected={props.response == "declined"}
        >
          <Text class="text-fg">{t("ready.decline")}</Text>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaReadyProps>> = {
  default: {"title":"","detail":"Ranked · Foundry","timeText":"12s","players":[{"id":"rhea","name":"Rhea","state":"ready"},{"id":"kai","name":"Kai","state":"pending"},{"id":"ana","name":"Ana","state":"declined"}],"response":"","canRespond":true,"closed":false,"loading":false},
  selected: {"players":[{"id":"rhea","name":"Rhea","state":"ready"},{"id":"kai","name":"Kai","state":"ready"}],"response":"ready"},
  disabled: {"canRespond":false,"detail":"Only joined players can respond"},
  closed: {"closed":true,"detail":"One player declined. Returning to queue.","response":"declined"},
  loading: {"loading":true},
  empty: {"players":[],"detail":"Waiting for participants"},
  overflow: {"title":"Championship Series · Grand Final ready check","detail":"A response is needed from every player before the tournament can continue","players":[{"id":"long","name":"Maximilian Vandenberghe-Okonkwo","state":"pending"},{"id":"offline","name":"Abernathy-Whitcombe","state":"offline"},{"id":"rhea","name":"Rhea","state":"ready"},{"id":"kai","name":"Kai","state":"declined"}]},
};
