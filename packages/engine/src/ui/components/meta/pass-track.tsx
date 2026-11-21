import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import { EmptyState } from "../base/empty-state";
import { ErrorState } from "../base/error-state";
import { Spinner } from "../base/spinner";
import "./meta.en.ogt";

export interface PassTrackProps {
  /** Heading; empty uses Season pass. */
  title?: string;
  /** The game's season name, a kicker over the title. */
  season?: string;
  /** The game's already formatted time remaining, after the season in the kicker; no clock runs here. */
  refresh?: string;
  /** The player's pass level as the game words it, on a hexagon beside the progress; empty hides it. */
  level?: string;
  /** Progress to the next tier, 0 to 1, supplied by the game. Default 0. */
  progress?: number;
  /** Progress words, such as 240 / 500 XP; empty uses a percent. */
  progressText?: string;
  /** The game owns the premium entitlement. Without it every premium reward reads Locked. Default false. */
  premiumUnlocked?: boolean;
  /** Optional upgrade action while premium is not unlocked (Get premium); no purchase logic. */
  upgradeLabel?: string;
  /** Keyed rewards in the game's order: { id, tier, premium, name, icon, amount, detail, state }. state locked/available/claimed. Premium rewards fill the premium lane, the rest the free lane; each reward is one tile, and its tier and state are always written in words. Default []. */
  rewards?: ReadonlyArray<{ id: string; tier: string; premium: boolean; name: string; icon: string; amount: string; detail: string; state: string }>;
  /** Loading the track; no reward action is available. Default false. */
  loading?: boolean;
  /** A failure explanation with retry; empty is no error. */
  error?: string;
  /** Prevent claims and upgrades while the game processes an operation. Default false. */
  disabled?: boolean;
  /** The claim event, with the row's key where it comes from a row. */
  onClaim?: (e: UiEvent, key?: string) => void;
  /** The upgrade event, with the row's key where it comes from a row. */
  onUpgrade?: (e: UiEvent, key?: string) => void;
}

/**
 * A season pass: a kicker with the season and the time left over the
 * title, a gold Get premium while premium is not owned, the level on a
 * hexagon beside a leaning bar to the next tier, and two lanes of tier
 * tiles, premium (in gold) over free. Each lane scrolls sideways on its
 * own; on a narrow screen its name sits above it. A tile says its tier,
 * shows the reward and how much, and ends in Claim when it is ready and
 * owned, Claimed once taken, or Locked (a premium tile always reads
 * Locked without premium). The game owns progress, entitlement and every
 * reward's state; the track reports claim (a reward's key), upgrade and
 * retry.
 */
export function PassTrack(props: PassTrackProps) {
  return (
    <View class={`grow shrink basis-0 border-rim nine-panel-cut bg-glass-hi text-fg w-full p-4 gap-3 border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title || t("meta.pass.title")} busy={(props.loading ?? false)}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class={`flex-row flex-wrap items-end min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <View class="grow shrink basis-40 gap-0.5 compact:basis-full">
          <Show when={props.season || props.refresh}>
            <Text class={`text-xs font-semibold tracking-kicker uppercase text-fg-subtle`}>
              {props.season && props.refresh ? t("meta.pass.kicker", { season: props.season, refresh: props.refresh }) : props.season || props.refresh}
            </Text>
          </Show>
          <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg m-0`}>
            {props.title || t("meta.pass.title")}
          </Text>
        </View>
        <Show when={!(props.premiumUnlocked ?? false) && props.upgradeLabel}>
          <View class={`flex-row items-center justify-center gap-1 text-accent max-w-full min-h-11 compact:w-full`} role="button" disabled={(props.disabled ?? false) || (props.loading ?? false) || !!props.error} onPress={(e) => props.onUpgrade?.(e)}>
            <Text class="whitespace-normal">{props.upgradeLabel}</Text>
          </View>
        </Show>
      </View>
      <Show when={(props.loading ?? false)} fallback={
          <Show when={props.error} fallback={
              <Show when={!(props.rewards ?? []).length} fallback={
                  <Scroll class="grow shrink basis-0 w-full gap-3">
                    <View class={`flex-row items-center w-full gap-3`}>
                      <Show when={props.level}>
                        <View class={`justify-center items-center text-title font-bold text-hud-accent-fg bg-linear-to-b from-hud-accent-hi to-hud-accent gap-2 size-16`} label={t("meta.pass.level", { level: props.level })}>
                          <Text>{props.level}</Text>
                        </View>
                      </Show>
                      <View class="grow shrink basis-0 gap-1">
                        <View class="flex-row items-baseline w-full flex-wrap gap-1">
                          <Text class={`grow shrink basis-0 font-semibold text-xs leading-4.5 text-fg-subtle tracking-kicker uppercase`}>
                            {t("meta.pass.progress")}
                          </Text>
                          <Text class={`tabular-nums text-fg whitespace-nowrap font-bold`}>
                            {props.progressText || `${formatNumber((props.progress ?? 0) * 100, { decimals: 0 })}%`}
                          </Text>
                        </View>
                        <View class={`bg-track shadow-sunk w-full h-2.5 mx-1 rounded-capsule`} role="meter" label={t("meta.pass.progress")} valueText={props.progressText || `${formatNumber((props.progress ?? 0) * 100, { decimals: 0 })}%`}>
                          <View class={`bg-hud-accent absolute left-0 inset-y-0`} style={{ width: `${(100 * ((props.progress ?? 0) > 1 ? 1 : (props.progress ?? 0) < 0 ? 0 : (props.progress ?? 0)))}%` }} />
                        </View>
                      </View>
                    </View>
                    <View class={`grid grid-cols-[7.857rem_minmax(0,1fr)] w-full gap-2 compact:grid-cols-1`} role="group" label={t("meta.pass.rewards")}>
                      <View class="justify-center pl-2 font-bold text-fg-muted border-l-[3.424px] border-fg-subtle gap-0.5 compact:flex-row compact:justify-start compact:items-baseline compact:py-0.5 compact:gap-1 data-lane-premium:text-gold data-lane-premium:border-l-[3.424px] data-lane-premium:border-gold" data-lane-premium={true}>
                        <Text>{t("meta.pass.premium")}</Text>
                        <Show when={!(props.premiumUnlocked ?? false)}>
                          <Text class={`text-caption text-fg-subtle`}>
                            {t("meta.locked")}
                          </Text>
                        </Show>
                      </View>
                      <Scroll class={`flex-row items-center pr-12 pl-0.5 py-0.5 m-0 gap-1`} label={t("meta.pass.premiumRewards")} data-lane-premium={true}>
                        <For each={(props.rewards ?? [])}>
                          {(r) => (
                          <Show when={r().premium}>
                            <View class={`group/reward grow basis-31 min-w-31 items-center text-center bg-tile border border-rim nine-plate-cut p-1 gap-1 data-premium:bg-tile`} label={t("meta.pass.reward", { tier: r().tier, name: r().name, state: r().state === 'claimed' ? t("meta.claimed") : r().state === 'available' && (props.premiumUnlocked ?? false) ? t("meta.available") : t("meta.locked") })} data-premium={true} data-not-ready={!(r().state === 'available' && (props.premiumUnlocked ?? false))} data-ready={r().state === 'available' && (props.premiumUnlocked ?? false)} data-state-claimed={(r().state) === 'claimed'}>
                              <Text class={`self-start text-xs font-bold tracking-kicker uppercase text-fg-subtle whitespace-nowrap`}>
                                {t("meta.pass.tier", { tier: r().tier })}
                              </Text>
                              <Icon class={`size-14.5 group-data-state-claimed/reward:opacity-45`} name={r().icon} />
                              <Text class={`text-fg max-w-full font-semibold`}>
                                {r().name}
                              </Text>
                              <Show when={r().amount}>
                                <Text class={`font-bold text-fg whitespace-nowrap`}>
                                  {r().amount}
                                </Text>
                              </Show>
                              <Show when={r().detail}>
                                <Text class={`max-w-full text-xs text-fg-muted m-0`}>
                                  {r().detail}
                                </Text>
                              </Show>
                              <Show when={r().state === 'claimed'} fallback={
                                  <Show when={r().state === 'available' && (props.premiumUnlocked ?? false)} fallback={
                                      <View class={`flex-row items-center mt-auto text-xs font-bold tracking-label uppercase text-fg-muted nine-chip-cut py-0.5 px-1 gap-0.5 ${packClass("Chip", "frame")}`}>
                                        <Text class="whitespace-nowrap">{t("meta.locked")}</Text>
                                      </View>
                                  }>
                                    <View class={`flex-row items-center justify-center gap-1 text-accent mt-auto w-full min-h-11`} role="button" disabled={(props.disabled ?? false)} onPress={(e) => props.onClaim?.(e, r().id)}>
                                      <Text class="whitespace-normal">{t("meta.claim")}</Text>
                                    </View>
                                  </Show>
                              }>
                                <View class={`flex-row items-center mt-auto text-xs font-bold tracking-label uppercase text-fg-muted nine-chip-cut py-0.5 px-1 gap-0.5 data-kind-claimed:bg-none data-kind-claimed:bg-success data-kind-claimed:text-boon-fg ${packClass("Chip", "frame")}`} data-kind-claimed={true}>
                                  <Icon class="size-3" name="check" />
                                  <Text class="whitespace-nowrap">{t("meta.claimed")}</Text>
                                </View>
                              </Show>
                            </View>
                          </Show>
                          )}
                        </For>
                      </Scroll>
                      <View class="justify-center pl-2 font-bold text-fg-muted border-l-[3.424px] border-fg-subtle gap-0.5 compact:flex-row compact:justify-start compact:items-baseline compact:py-0.5 compact:gap-1">
                        <Text>{t("meta.pass.free")}</Text>
                      </View>
                      <Scroll class={`flex-row items-center pr-12 pl-0.5 py-0.5 m-0 gap-1`} label={t("meta.pass.freeRewards")}>
                        <For each={(props.rewards ?? [])}>
                          {(r) => (
                          <Show when={!r().premium}>
                            <View class={`group/reward grow basis-31 min-w-31 items-center text-center bg-tile border border-rim nine-plate-cut p-1 gap-1 data-premium:bg-tile`} label={t("meta.pass.reward", { tier: r().tier, name: r().name, state: r().state === 'claimed' ? t("meta.claimed") : r().state === 'available' ? t("meta.available") : t("meta.locked") })} data-premium={false} data-not-ready={!(r().state === 'available')} data-ready={r().state === 'available'} data-state-claimed={(r().state) === 'claimed'}>
                              <Text class={`self-start text-xs font-bold tracking-kicker uppercase text-fg-subtle whitespace-nowrap`}>
                                {t("meta.pass.tier", { tier: r().tier })}
                              </Text>
                              <Icon class={`size-14.5 group-data-state-claimed/reward:opacity-45`} name={r().icon} />
                              <Text class={`text-fg max-w-full font-semibold`}>
                                {r().name}
                              </Text>
                              <Show when={r().amount}>
                                <Text class={`font-bold text-fg whitespace-nowrap`}>
                                  {r().amount}
                                </Text>
                              </Show>
                              <Show when={r().detail}>
                                <Text class={`max-w-full text-xs text-fg-muted m-0`}>
                                  {r().detail}
                                </Text>
                              </Show>
                              <Show when={r().state === 'claimed'} fallback={
                                  <Show when={r().state === 'available'} fallback={
                                      <View class={`flex-row items-center mt-auto text-xs font-bold tracking-label uppercase text-fg-muted nine-chip-cut py-0.5 px-1 gap-0.5 ${packClass("Chip", "frame")}`}>
                                        <Text class="whitespace-nowrap">{t("meta.locked")}</Text>
                                      </View>
                                  }>
                                    <View class={`flex-row items-center justify-center gap-1 text-accent mt-auto w-full min-h-11`} role="button" disabled={(props.disabled ?? false)} onPress={(e) => props.onClaim?.(e, r().id)}>
                                      <Text class="whitespace-normal">{t("meta.claim")}</Text>
                                    </View>
                                  </Show>
                              }>
                                <View class={`flex-row items-center mt-auto text-xs font-bold tracking-label uppercase text-fg-muted nine-chip-cut py-0.5 px-1 gap-0.5 data-kind-claimed:bg-none data-kind-claimed:bg-success data-kind-claimed:text-boon-fg ${packClass("Chip", "frame")}`} data-kind-claimed={true}>
                                  <Icon class="size-3" name="check" />
                                  <Text class="whitespace-nowrap">{t("meta.claimed")}</Text>
                                </View>
                              </Show>
                            </View>
                          </Show>
                          )}
                        </For>
                      </Scroll>
                    </View>
                  </Scroll>
              }>
                <EmptyState title={t("meta.pass.empty")} icon="bag" />
              </Show>
          }>
            <ErrorState body={props.error ?? ''} />
          </Show>
      }>
        <View class="flex-row items-center p-4 gap-2" role="status">
          <Spinner name={t("meta.loading")} />
          <Text class="shrink">{t("meta.loading")}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<PassTrackProps>> = {
  default: {"title":"","season":"Season of Embers","refresh":"Ends in 12 days","level":"3","progress":0.48,"progressText":"240 / 500 XP","premiumUnlocked":false,"upgradeLabel":"Get premium","rewards":[{"id":"shard","tier":"1","premium":true,"name":"Ember shard","icon":"fire","amount":"×5","detail":"","state":"available"},{"id":"cloak","tier":"2","premium":true,"name":"Ashen Cloak","icon":"shield","amount":"Look","detail":"","state":"available"},{"id":"flame","tier":"3","premium":true,"name":"Phoenix flame","icon":"fire","amount":"×2","detail":"","state":"available"},{"id":"blade","tier":"4","premium":true,"name":"Ember Blade","icon":"sword","amount":"Skin","detail":"Legendary weapon skin","state":"locked"},{"id":"crown","tier":"5","premium":true,"name":"Sun Crown","icon":"crown","amount":"×1","detail":"","state":"locked"},{"id":"stones","tier":"6","premium":true,"name":"Moonstones","icon":"gem","amount":"×20","detail":"","state":"locked"},{"id":"gold","tier":"1","premium":false,"name":"Gold","icon":"coin","amount":"500","detail":"","state":"claimed"},{"id":"keys","tier":"2","premium":false,"name":"Chest keys","icon":"key","amount":"×3","detail":"","state":"claimed"},{"id":"potions","tier":"3","premium":false,"name":"Potions","icon":"potion","amount":"×5","detail":"","state":"available"},{"id":"leaf","tier":"4","premium":false,"name":"Silverleaf","icon":"leaf","amount":"×10","detail":"","state":"locked"},{"id":"map","tier":"5","premium":false,"name":"Scroll","icon":"map","amount":"×1","detail":"","state":"locked"},{"id":"gems","tier":"6","premium":false,"name":"Gems","icon":"gem","amount":"100","detail":"","state":"locked"}]},
  empty: {"rewards":[]},
  loading: {"loading":true},
  error: {"error":"The season service is unavailable. Your progress is saved."},
  premium: {"season":"Season of Embers","level":"50","premiumUnlocked":true,"progress":1,"progressText":"Season complete","rewards":[{"id":"skin","tier":"50","premium":true,"name":"Watcher of the Ninefold Gate","icon":"crown","amount":"×1","detail":"An exclusive outfit for completing this season’s premium track.","state":"available"},{"id":"chest","tier":"50","premium":false,"name":"Gold","icon":"coin","amount":"5,000","detail":"","state":"claimed"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"claim":"Claim the available reward by stable key","upgrade":"Inspect the premium upgrade","retry":"Retry loading the track"} as const;
