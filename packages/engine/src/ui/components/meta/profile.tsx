import type { Texture } from "@origingame/engine";
import {For, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Avatar } from "../base/avatar";
import { Spinner } from "../base/spinner";
import "./profile.en.ogt";

export interface ProfileProps {
  /** Player display name; empty uses Player. */
  name: string;
  /** Stable player handle or discriminator, as the game words it; empty hides it. */
  handle: string;
  /** Loaded portrait texture; zero uses the base Avatar fallback. */
  portrait: Texture | 0;
  /** Level shown by Avatar; zero hides it. */
  level: number;
  /** Avatar presence: online, away, busy or offline; empty hides it. */
  status: string;
  /** Game-formatted rank or title; empty hides it. */
  rank: string;
  /** Player biography or activity; wraps, empty hides it. */
  bio: string;
  /** Keyed { id, label, value, detail } readouts. The game formats all values and decides their order; detail is an optional explanation. */
  stats: ReadonlyArray<{ id: string; label: string; value: string; detail: string }>;
  /** Keyed { id, label, disabled, reason, pending } actions, e.g. friend, invite, edit or report. action reports the stable row id; pending disables it and reads Sending. The game owns permissions and effects. */
  actions: ReadonlyArray<{ id: string; label: string; disabled: boolean; reason: string; pending: boolean }>;
  /** Hide statistics and show Stats are private; no private values are mounted. */
  privateStats?: boolean;
  /** Loading profile data; hides all stale identity, statistics and actions behind the shared Spinner. */
  loading?: boolean;
  /** Error message; hides stale data and offers retry. Empty means no error. */
  error: string;
  /** No profile found; hides stale identity, statistics and actions. */
  empty?: boolean;
  /** The action event, with the row's key where it comes from a row. */
  onAction?: (e: UiEvent, key?: string) => void;
  /** The retry event, with the row's key where it comes from a row. */
  onRetry?: (e: UiEvent, key?: string) => void;
}

/**
 * A player profile: the avatar with level and presence, the name, handle,
 * rank and bio, the stats the game formats and the actions it offers
 * (friend, invite, report). Private stats, loading, an error and no profile
 * each have their state.
 */
export function Profile(props: ProfileProps) {
  return (
    <View class={`w-full p-4 gap-3 compact:p-2 ${packClass("Panel", "frame")}`} label={t("profile.title")} busy={props.loading}>
      <Show when={props.loading} fallback={
          <Show when={props.error} fallback={
              <Show when={props.empty} fallback={
                <>
                  <View class="gap-0">
                    <Scroll class="pb-4 gap-3">
                      <View class={`flex-row items-center gap-3`}>
                        <View class={``}>
                          <Avatar name={props.name || t("profile.player")} showName={false} portrait={props.portrait} level={props.level} status={props.status} />
                        </View>
                        <View class="grow shrink basis-0 items-start gap-1">
                          <Text class={`font-display text-3xl font-bold leading-flat tracking-sm uppercase text-fg max-w-full m-0 compact:text-xl`}>
                            {props.name || t("profile.player")}
                          </Text>
                          <Show when={props.handle}>
                            <View class={`text-caption leading-4.5 text-fg-muted max-w-full`}>
                              <Text>{props.handle}</Text>
                            </View>
                          </Show>
                          <Show when={props.rank}>
                            <View class={`font-bold text-xs leading-normal text-gold-fg max-w-full tracking-label uppercase bg-gold-hi rounded-control py-0 px-1`}>
                              <Text>{props.rank}</Text>
                            </View>
                          </Show>
                        </View>
                      </View>
                      <Show when={props.bio}>
                        <Text class={`text-reading leading-normal text-fg m-0`}>
                          {props.bio}
                        </Text>
                      </Show>
                      <Show when={props.privateStats} fallback={
                          <Show when={props.stats.length} fallback={
                              <Text class={`text-center text-base text-fg-muted m-0`}>
                                {t("profile.noStats")}
                              </Text>
                          }>
                            <View class={`grid grid-cols-fill-50 compact:grid-cols-1`} role="list" label={t("profile.stats")}>
                              <For each={props.stats}>
                                {(s) => (
                                <View class={`transition duration-control py-2 px-3 gap-1 data-selected:inset-ring-regular data-selected:inset-ring-accent data-done:inset-ring-regular data-done:inset-ring-success active:inset-ring-regular active:inset-ring-accent data-locked:scale-100 data-down:opacity-dim compact:py-1 compact:px-3 ${packClass("Plate", "frame")}`} role="listitem">
                                  <View class={`font-semibold text-xs leading-snug text-fg-muted tracking-label uppercase`}>
                                    <Text>{s().label}</Text>
                                  </View>
                                  <View class={`text-fg leading-flat font-bold compact:text-3xl`}>
                                    <Text class="tabular-nums">{s().value}</Text>
                                  </View>
                                  <Show when={s().detail}>
                                    <View class={`text-caption leading-4.5 text-fg-muted`}>
                                      <Text>{s().detail}</Text>
                                    </View>
                                  </Show>
                                </View>
                                )}
                              </For>
                            </View>
                          </Show>
                      }>
                        <Text class={`text-center text-base text-fg-muted m-0`}>
                          {t("profile.private")}
                        </Text>
                      </Show>
                    </Scroll>
                    <View class={`absolute bottom-0 h-4 pointer-events-none bg-linear-to-b from-transparent to-tile inset-x-0`} />
                  </View>
                  <Show when={props.actions.length}>
                    <View class={`flex-row flex-wrap items-center gap-2`}>
                      <For each={props.actions}>
                        {(a) => (
                        <View class="grow shrink basis-0 gap-1">
                          <View class={`flex-row items-center justify-center min-h-11 text-caption font-bold tracking-label uppercase text-center py-1 px-4 gap-1 disabled:opacity-dim ${packClass("Button", "plate")}`} role="button" disabled={a().disabled || a().pending} busy={a().pending} onPress={(e) => props.onAction?.(e, a().id)}>
                            <Text class="whitespace-normal">{a().pending ? t("profile.sending") : a().label}</Text>
                          </View>
                          <Show when={a().reason}>
                            <View class={`text-caption leading-4.5 text-fg-muted text-center`}>
                              <Text>{a().reason}</Text>
                            </View>
                          </Show>
                        </View>
                        )}
                      </For>
                    </View>
                  </Show>
                </>
              }>
                <Text class={`text-center text-base text-fg-muted m-0`} role="status">
                  {t("profile.empty")}
                </Text>
              </Show>
          }>
            <Text class={`text-center text-base text-fg-muted m-0`} role="status">
              {props.error}
            </Text>
            <View class={`flex-row items-center justify-center min-h-11 text-caption font-bold tracking-label uppercase text-center py-1 px-4 gap-1 ${packClass("Button", "plate")}`} role="button" onPress={(e) => props.onRetry?.(e)}>
              <Text class="whitespace-normal">{t("profile.retry")}</Text>
            </View>
          </Show>
      }>
        <View class={`flex-row items-center justify-center gap-2`}>
          <Spinner name={t("profile.loading")} />
        </View>
        <Text class={`text-center text-base text-fg-muted m-0`} role="status">
          {t("profile.loading")}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ProfileProps>> = {
  default: {"name":"Brenna Vale","handle":"@brenna","portrait":0,"level":12,"status":"online","rank":"Gold II · Season 4","bio":"Support main. Here for the team.","stats":[{"id":"wins","label":"Wins","value":"128","detail":"This season"},{"id":"matches","label":"Matches","value":"204","detail":"Ranked"},{"id":"best","label":"Best streak","value":"9","detail":"Consecutive wins"}],"actions":[{"id":"friend","label":"Add friend","disabled":false,"reason":"","pending":false},{"id":"invite","label":"Invite","disabled":true,"reason":"Player is in a match","pending":false}],"privateStats":false,"loading":false,"error":"","empty":false},
  private: {"privateStats":true,"status":"offline"},
  pending: {"actions":[{"id":"friend","label":"Add friend","disabled":false,"reason":"","pending":true}]},
  loading: {"loading":true},
  error: {"error":"Profile could not be loaded. Check your connection."},
  empty: {"empty":true},
  "no-stats": {"stats":[],"actions":[]},
  overflow: {"name":"Maximilian Vandenberghe-Okonkwo","handle":"@the-unforgotten-champion-of-the-eastern-isles","rank":"Grandmaster · Northern Championship Series","bio":"A player biography that takes several lines on a phone and never hides the player actions or pushes statistics beyond the screen width.","stats":[{"id":"damage","label":"Highest accumulated damage in a single championship","value":"123,456,789,012","detail":"All seasons combined"},{"id":"wins","label":"Wins","value":"12,345","detail":"Across all regions"}],"actions":[{"id":"friend","label":"Send a companion request","disabled":false,"reason":"","pending":false},{"id":"invite","label":"Invite to the championship lobby","disabled":true,"reason":"This player does not accept invitations from non-friends","pending":false}]},
};
