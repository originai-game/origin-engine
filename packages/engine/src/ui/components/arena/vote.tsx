import { For, Icon, Scroll, Show, Text, View, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./vote.en.strings.ts";

export interface VoteOption {
  id: string;
  /** What the map or choice is called. */
  name: string;
  /** An icon of the style's atlas; empty is none. */
  icon?: string;
  /** A line about it. */
  detail?: string;
  /** The votes so far; the game counts them. */
  votes?: number;
  /** It cannot be chosen. */
  disabled?: boolean;
  /** Why it cannot be chosen (Not installed). */
  reason?: string;
}

export interface ArenaVoteProps {
  /** Heading; empty uses Map vote. */
  title?: string;
  /** Voting rules or a disabled reason, supplied by the game. */
  detail?: string;
  /** Game-owned time remaining; empty hides it. Hidden once closed. */
  timeText?: string;
  /** The choices. choose reports the stable id, never the index. */
  options?: ReadonlyArray<VoteOption>;
  /** The local vote's option id; empty means no vote. A click reports choose but does not change it locally. */
  selected?: string;
  /** The winning option id, supplied by the game. Shown only when closed; empty means no announced result. */
  winner?: string;
  /** Whether the local player can vote or change a vote. Default true. */
  canVote?: boolean;
  /** Voting is over; options remain readable but disabled. */
  closed?: boolean;
  /** Waiting for options; hides stale options and disables voting. */
  loading?: boolean;
  /** Choose an option. */
  onChoose?: (e: UiEvent, id: string) => void;
}

/**
 * The vote on a smoked cut card: the clock in gold, an option a cut tile
 * of dark glass with its tally at the end, the player's own ringed in the
 * accent and washed in it. The game counts votes and decides winners;
 * choose reports the option's stable id and changes nothing locally.
 * Approximations: the chosen tile's wash is flat and keeps no rim (a cut
 * takes one fill), and the scroll fade at the list's foot is dropped.
 */
const CARD = `w-full max-h-full p-4 compact:p-2 ${packClass("Panel", "frame")}`;

/**
 * A map or mode vote: the options, the local vote, the time left and, once
 * closed, the winner the game names. It reports onChoose with the option's
 * id; the game counts the votes.
 */
export function ArenaVote(props: ArenaVoteProps) {
  const heading = () => (props.title ?? "") != "" ? props.title : t("vote.title");
  return (
    <View class={`flex-col gap-y-2 text-fg ${CARD}`} label={heading()} busy={props.loading}>
      <Text class="text-title leading-tight font-bold uppercase tracking-sm text-fg text-center">{heading()}</Text>
      <Show when={(props.detail ?? "") != ""}>
        <Text class="text-caption font-semibold text-fg-muted">{props.detail}</Text>
      </Show>
      <Show when={props.closed} fallback={
        <Show when={(props.timeText ?? "") != ""}>
          <Text class="text-title leading-tight font-bold tabular-nums text-gold">{props.timeText}</Text>
        </Show>
      }>
        <Text class="text-xs leading-normal font-bold tracking-label uppercase text-fg-muted" role="status">{t("vote.closed")}</Text>
      </Show>
      <Show when={props.loading} fallback={
        <Show when={(props.options ?? []).length}>
          <Scroll class="min-h-0">
            <View class="flex-col gap-y-1 pb-4" role="group" label={t("vote.options")}>
              <For each={props.options ?? []} key={(o) => o.id}>
                {(o) => (
                  <View
                    class={`group/vote flex-row flex-wrap items-center gap-x-2 gap-y-1 w-full min-h-11 py-2 px-3 text-start rounded-card bg-glass-hi hover:bg-raised-hi data-selected:bg-surface-hover data-selected:border-l-thick data-selected:border-hud-accent disabled:text-fg-muted transition duration-control compact:py-1 compact:px-2`}
                    role="button"
                    label={o().name}
                    pressed={props.selected == o().id}
                    disabled={!(props.canVote ?? true) || props.closed || o().disabled}
                    onPress={(e) => props.onChoose?.(e, o().id)}
                    data-selected={props.selected == o().id}
                    data-winner={props.closed && props.winner == o().id}
                  >
                    <Show when={(o().icon ?? "") != ""}>
                      <Icon class="shrink-0 size-6 text-fg" name={o().icon} />
                    </Show>
                    <View class="flex-1 min-w-0 flex-col items-start gap-y-0.25">
                      <Text class="w-full text-lg leading-tight font-bold text-fg">{o().name}</Text>
                      <Show when={(o().detail ?? "") != ""}>
                        <Text class="text-caption text-fg-muted">{o().detail}</Text>
                      </Show>
                      <Show when={props.selected == o().id}>
                        <Text class="text-xs leading-normal font-bold tracking-label uppercase text-hud-accent-ink">{t("vote.selected")}</Text>
                      </Show>
                      <Show when={props.closed && props.winner == o().id}>
                        <Text class="text-xs leading-normal font-bold tracking-label uppercase text-success">{t("vote.winner")}</Text>
                      </Show>
                      <Show when={o().disabled && (o().reason ?? "") != ""}>
                        <Text class="text-xs leading-normal font-bold tracking-label uppercase text-fg-muted">{o().reason}</Text>
                      </Show>
                    </View>
                    <Text class="shrink basis-auto text-base font-bold tabular-nums text-fg compact:basis-full compact:text-end">{t("vote.tally", { count: formatNumber(o().votes ?? 0) })}</Text>
                  </View>
                )}
              </For>
            </View>
          </Scroll>
        </Show>
      }>
        <Text class="text-caption font-semibold text-fg-muted" role="status">{t("vote.loading")}</Text>
      </Show>
      <Show when={!props.loading && (props.options ?? []).length == 0}>
        <Text class="text-caption font-semibold text-fg-muted" role="status">{t("vote.empty")}</Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaVoteProps>> = {
  default: {"title":"","detail":"Choose the next map. You can change your vote.","timeText":"18s","options":[{"id":"foundry","name":"Foundry","icon":"fire","detail":"Close quarters · 2 v 2","votes":3,"disabled":false,"reason":""},{"id":"grove","name":"Moonlit Grove","icon":"leaf","detail":"Open lanes · 2 v 2","votes":1,"disabled":false,"reason":""},{"id":"citadel","name":"Citadel","icon":"crown","detail":"Tournament map","votes":0,"disabled":true,"reason":"Not installed"}],"selected":"foundry","winner":"","canVote":true,"closed":false,"loading":false},
  closed: {"closed":true,"winner":"grove","detail":"Moonlit Grove won the vote.","options":[{"id":"foundry","name":"Foundry","icon":"fire","detail":"Close quarters · 2 v 2","votes":3,"disabled":false,"reason":""},{"id":"grove","name":"Moonlit Grove","icon":"leaf","detail":"Open lanes · 2 v 2","votes":5,"disabled":false,"reason":""},{"id":"citadel","name":"Citadel","icon":"crown","detail":"Tournament map","votes":0,"disabled":true,"reason":"Not installed"}]},
  disabled: {"canVote":false,"detail":"Spectators cannot vote","selected":""},
  loading: {"loading":true},
  empty: {"options":[],"selected":"","detail":"Waiting for the host to choose maps"},
  overflow: {"title":"Championship Series · Deciding map selection","options":[{"id":"isles","name":"The Shattered Isles of the Unforgotten Ancients","icon":"map","detail":"An extended description that must remain readable without pushing a vote count off the phone","votes":1234567,"disabled":false,"reason":""},{"id":"grove","name":"Moonlit Grove","icon":"leaf","detail":"Open lanes","votes":99,"disabled":false,"reason":""}],"selected":"isles"},
};
