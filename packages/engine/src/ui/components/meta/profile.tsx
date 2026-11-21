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

export function Profile(props: ProfileProps) {
  return (
    <View class={`nine-panel-cut bg-glass/96 border border-white/14 text-fg w-full p-4 gap-3 compact:p-2`} label={t("profile.title")} busy={props.loading}>
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
                          <Text class={`font-display text-3xl font-bold leading-[1.05] tracking-sm uppercase text-fg max-w-full m-0 compact:text-xl`}>
                            {props.name || t("profile.player")}
                          </Text>
                          <Show when={props.handle}>
                            <View class={`text-caption leading-4.5 text-fg/66 max-w-full`}>
                              <Text>{props.handle}</Text>
                            </View>
                          </Show>
                          <Show when={props.rank}>
                            <View class={`font-bold text-xs leading-[1.5] text-gold-fg max-w-full tracking-[0.14em] uppercase bg-gold-hi nine-chip-cut py-0 px-1 ${packClass("Chip", "frame")}`}>
                              <Text>{props.rank}</Text>
                            </View>
                          </Show>
                        </View>
                      </View>
                      <Show when={props.bio}>
                        <Text class={`text-reading leading-[1.4] text-fg m-0`}>
                          {props.bio}
                        </Text>
                      </Show>
                      <Show when={props.privateStats} fallback={
                          <Show when={props.stats.length} fallback={
                              <Text class={`text-center text-base text-fg/66 m-0`}>
                                {t("profile.noStats")}
                              </Text>
                          }>
                            <View class={`grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] compact:grid-cols-[minmax(0,1fr)]`} role="list" label={t("profile.stats")}>
                              <For each={props.stats}>
                                {(s) => (
                                <View class={`nine-plate-cut bg-plate-hi border border-white/18 text-fg transition duration-317 py-2 px-3 gap-1 data-selected:border-accent data-done:border-success data-active:border-accent active:scale-96 data-locked:scale-100 disabled:scale-100 data-down:opacity-45 compact:py-1 compact:px-3`} role="listitem">
                                  <View class={`font-semibold text-xs leading-snug text-fg/78 tracking-[0.14em] uppercase`}>
                                    <Text>{s().label}</Text>
                                  </View>
                                  <View class={`text-fg leading-[1.05] font-bold compact:text-3xl`}>
                                    <Text class="tabular-nums">{s().value}</Text>
                                  </View>
                                  <Show when={s().detail}>
                                    <View class={`text-caption leading-4.5 text-fg/66`}>
                                      <Text>{s().detail}</Text>
                                    </View>
                                  </Show>
                                </View>
                                )}
                              </For>
                            </View>
                          </Show>
                      }>
                        <Text class={`text-center text-base text-fg/66 m-0`}>
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
                          <View class={`flex-row items-center justify-center min-h-11 text-caption font-bold tracking-[0.14em] uppercase text-fg bg-raised-lo/92 border border-white/24 nine-plate-cut text-center py-1 px-4 gap-1 hover:bg-raised-hi/95 hover:bg-transparent disabled:opacity-45`} role="button" disabled={a().disabled || a().pending} busy={a().pending} onPress={(e) => props.onAction?.(e, a().id)}>
                            <Text class="whitespace-normal">{a().pending ? t("profile.sending") : a().label}</Text>
                          </View>
                          <Show when={a().reason}>
                            <View class={`text-caption leading-4.5 text-fg/66 text-center`}>
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
                <Text class={`text-center text-base text-fg/66 m-0`} role="status">
                  {t("profile.empty")}
                </Text>
              </Show>
          }>
            <Text class={`text-center text-base text-fg/66 m-0`} role="status">
              {props.error}
            </Text>
            <View class={`flex-row items-center justify-center min-h-11 text-caption font-bold tracking-[0.14em] uppercase text-fg bg-raised-lo/92 border border-white/24 nine-plate-cut text-center py-1 px-4 gap-1 hover:bg-raised-hi/95 hover:bg-transparent`} role="button" onPress={(e) => props.onRetry?.(e)}>
              <Text class="whitespace-normal">{t("profile.retry")}</Text>
            </View>
          </Show>
      }>
        <View class={`flex-row items-center justify-center gap-2`}>
          <Spinner name={t("profile.loading")} />
        </View>
        <Text class={`text-center text-base text-fg/66 m-0`} role="status">
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
