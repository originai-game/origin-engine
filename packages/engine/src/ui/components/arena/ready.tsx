import { For, Scroll, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
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
 * local readiness is faked. Approximations: the scroll fade at the
 * list's foot is dropped.
 */
const CARD = `w-full max-h-full p-4 nine-panel-cut bg-plate-hi ${packClass("Panel", "frame")}`;
const BUTTON = `flex-1 basis-44 flex-row items-center justify-center gap-x-1 min-h-11 px-4 text-caption font-bold tracking-label uppercase text-fg nine-plate-cut bg-glass-hi hover:bg-raised-hi disabled:opacity-45 ${packClass("Button", "plate")}`;

/**
 * A ready check: the players asked and their answers, the time left as the
 * game words it, and Accept and Decline for the local player. It never
 * starts or expires a timer; the game closes the check.
 */
export function ArenaReady(props: ArenaReadyProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("ready.title");
  const canAct = () => (props.canRespond ?? true) && !props.closed && !props.loading && (props.players ?? []).length > 0;
  return (
    <View class={`flex-col gap-y-2 text-fg ${CARD}`} label={heading()} busy={props.loading}>
      <Text class="text-title leading-tight font-bold uppercase tracking-sm text-fg text-center">{heading()}</Text>
      <Show when={(props.detail ?? "") != ""}>
        <Text class="text-caption font-semibold text-fg-muted text-center">{props.detail}</Text>
      </Show>
      <Show when={props.closed} fallback={
        <Show when={(props.timeText ?? "") != ""}>
          <Text class="text-7xl leading-none font-bold tabular-nums text-gold text-center">{props.timeText}</Text>
        </Show>
      }>
        <Text class="text-caption font-bold tracking-label uppercase text-fg-muted text-center" role="status">{t("ready.closed")}</Text>
      </Show>
      <Show when={props.loading} fallback={
        <Show when={(props.players ?? []).length}>
          <Scroll class="min-h-0">
            <View class="flex-col gap-y-1 pb-4" role="list" label={t("ready.players")}>
              <For each={props.players ?? []} key={(p) => p.id}>
                {(p) => (
                  <View
                    class={`group/readyrow flex-row flex-wrap items-center justify-between gap-x-2 min-h-8 py-1 px-3 ${packClass("Row", "frame")} data-ready:border-l-3 data-ready:border-success data-declined:border-l-3 data-declined:border-danger`}
                    role="listitem"
                    data-ready={p().state == "ready"}
                    data-declined={p().state == "declined"}
                  >
                    <Text class={`flex-1 basis-16 min-w-0 text-reading font-semibold text-fg data-offline:text-fg-muted`} data-offline={p().state == "offline"}>{p().name}</Text>
                    <Text
                      class={`ml-auto text-caption leading-snug font-bold tracking-label uppercase text-fg-muted text-center data-ready:text-success data-declined:text-danger`}
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
        <Text class="text-caption font-semibold text-fg-muted text-center" role="status">{t("ready.loading")}</Text>
      </Show>
      <Show when={!props.loading && (props.players ?? []).length == 0}>
        <Text class="text-caption font-semibold text-fg-muted text-center" role="status">{t("ready.empty")}</Text>
      </Show>
      <View class="flex-row flex-wrap gap-x-1 gap-y-1">
        <View
          class={`flex-row items-center justify-center gap-1 ${BUTTON} text-boon-fg bg-none bg-success hover:bg-success ${packClass("Button", "plate", "primary")}`}
          role="button"
          label={t("ready.accept")}
          pressed={props.response == "ready"}
          disabled={!canAct()}
          onPress={(e) => props.onAccept?.(e)}
          data-selected={props.response == "ready"}
        >
          <Text>{t("ready.accept")}</Text>
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
  overflow: {"title":"Championship Series · Grand Final ready check","detail":"A response is needed from every player before the tournament can continue","players":[{"id":"long","name":"Maximilian Vandenberghe-Okonkwo","state":"pending"},{"id":"offline","name":"Abernathy-Whitcombe","state":"offline"},{"id":"rhea","name":"Rhea","state":"ready"},{"id":"kai","name":"Kai","state":"declined"}]},
};
