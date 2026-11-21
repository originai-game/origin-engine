import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { EmptyState } from "../base/empty-state";
import { ErrorState } from "../base/error-state";
import { Spinner } from "../base/spinner";
import "./meta.en.ogt";

export interface RevealProps {
  /** Heading; empty uses Rewards. */
  title?: string;
  /** A small line over the title naming where the rewards come from (Expedition rewards); empty hides it. */
  kicker?: string;
  /** Optional explanation under the sealed prize. */
  body?: string;
  /** The sealed prize's icon or picture (a chest, a pouch); empty uses the bag. */
  icon?: string;
  /** Game-owned sealed/loading/revealing/revealed. No roll, timer or reward mutation runs here. Default "sealed". */
  state?: string;
  /** Keyed { id, name, icon, amount, rarity, fresh } rewards, shown only once revealed. rarity is common/uncommon/rare/epic/legendary/mythic, in words and as the socket's colour; amount (×3) sits in the socket's corner; fresh marks a new one with a dot. Default []. */
  rewards?: ReadonlyArray<{ id: string; name: string; icon: string; amount: string; rarity: string; fresh: boolean }>;
  /** Offer skip while revealing; the game decides what skipping does. Default true. */
  canSkip?: boolean;
  /** Continue wording; empty uses Continue. */
  continueLabel?: string;
  /** Prevent opening, skipping, inspecting and continuing. Default false. */
  disabled?: boolean;
  /** A failed reveal with retry; rewards stay hidden until the game resolves it. */
  error?: string;
  /** The continue event, with the row's key where it comes from a row. */
  onContinue?: (e: UiEvent, key?: string) => void;
  /** The inspect event, with the row's key where it comes from a row. */
  onInspect?: (e: UiEvent, key?: string) => void;
  /** The open event, with the row's key where it comes from a row. */
  onOpen?: (e: UiEvent, key?: string) => void;
  /** The skip event, with the row's key where it comes from a row. */
  onSkip?: (e: UiEvent, key?: string) => void;
}

/**
 * A reward reveal: a kicker naming the source over the title; sealed, the
 * prize large over a gold glow with the game's words under it and Open;
 * revealing, a spinner and Skip; revealed, each reward in a socket of its
 * rarity (its count in a corner, a dot when it is new) with the rarity in
 * words over its name, and Continue. Rewards stay hidden until the game
 * says they are revealed; a reward is a button that reports inspect with
 * its key. The game owns the roll, the timing and the rewards; the panel
 * reports open, skip, inspect, continue and retry.
 */
export function Reveal(props: RevealProps) {
  return (
    <View class={`grow shrink basis-0 border-rim nine-panel-cut bg-glass-hi text-fg w-full p-4 gap-3 border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title || t("meta.reveal.title")} busy={(props.state ?? "sealed") === 'loading' || (props.state ?? "sealed") === 'revealing'}>
      <View class="absolute left-4 right-[30%] top-0 h-px bg-linear-to-r from-white/55 to-transparent to-70% pointer-events-none" />
      <View class={`min-h-12 border-b border-rim-soft w-full items-center text-center py-1 px-4 gap-0.5`}>
        <Show when={props.kicker}>
          <Text class={`text-xs font-semibold tracking-[0.18em] uppercase text-fg-subtle`}>
            {props.kicker}
          </Text>
        </Show>
        <Text class={`font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title || t("meta.reveal.title")}
        </Text>
      </View>
      <Show when={props.error} fallback={
          <Show when={(props.state ?? "sealed") === 'loading' || (props.state ?? "sealed") === 'revealing'} fallback={
              <Show when={(props.state ?? "sealed") === 'revealed'} fallback={
                <>
                  <View class={`grow shrink basis-0 justify-center items-center w-full gap-2`}>
                    <Icon class="size-4" name={props.icon || 'bag'} />
                  </View>
                  <Show when={props.body}>
                    <Text class={`text-caption text-fg-muted self-center max-w-105 text-center m-0`}>
                      {props.body}
                    </Text>
                  </Show>
                </>
              }>
                <Scroll class="grow shrink basis-0 gap-2">
                  <View class={`flex-wrap justify-center w-full p-1 m-0 gap-3 compact:grid compact:grid-cols-2`} label={props.title || t("meta.reveal.title")}>
                    <Show when={((props.rewards ?? []))?.length ?? 0} fallback={<View class="grow shrink basis-0"><EmptyState title={t("meta.reveal.empty")} icon="bag" /></View>}><For each={(props.rewards ?? [])}>
                      {(reward) => (
                      <View class="shrink basis-[8.571rem]">
                        <View class={`group/reward w-full p-0 hover:bg-transparent hover:bg-none hover:shadow-none`} role="button" disabled={(props.disabled ?? false)} label={t("meta.reveal.reward", { name: reward().name, rarity: reward().rarity === 'mythic' ? t("meta.rarity.mythic") : reward().rarity === 'legendary' ? t("meta.rarity.legendary") : reward().rarity === 'epic' ? t("meta.rarity.epic") : reward().rarity === 'rare' ? t("meta.rarity.rare") : reward().rarity === 'uncommon' ? t("meta.rarity.uncommon") : t("meta.rarity.common") })} onPress={(e) => props.onInspect?.(e, reward().id)} data-tier-common={(reward().rarity || 'common') === "common"}>
                          <View class="w-full items-center text-center gap-1">
                            <View class={`grow shrink basis-0 justify-center items-center w-[8.571rem] max-w-full aspect-square bg-tile border-[1.5px] border-border-strong nine-plate-cut gap-2 compact:w-[6.857rem] ${packClass("Slot", "frame")}`}>
                              <Icon class="size-[70%]" name={reward().icon} />
                              <View class="absolute bottom-0 h-0.75 text-border-strong bg-current inset-x-[0.571rem]" />
                              <Show when={reward().amount}>
                                <Text class={`absolute right-1 bottom-2 text-xs font-bold leading-[1.35] whitespace-nowrap text-fg bg-[#060910e0] nine-chip-cut py-0 px-0.5 ${packClass("Chip", "frame")}`}>
                                  {reward().amount}
                                </Text>
                              </Show>
                              <Show when={reward().fresh}>
                                <View class={`bg-[#38e8ff] text-fg absolute right-2 top-2 shadow-[0px_0px_9.136px_#38e8ff,0px_0px_0px_1.5px_#060910e5] size-1 rounded-capsule`} label={t("meta.new")} />
                              </Show>
                            </View>
                            <Text class={`text-xs font-bold tracking-[0.18em] uppercase text-border-strong`}>
                              {reward().rarity === 'mythic' ? t("meta.rarity.mythic") : reward().rarity === 'legendary' ? t("meta.rarity.legendary") : reward().rarity === 'epic' ? t("meta.rarity.epic") : reward().rarity === 'rare' ? t("meta.rarity.rare") : reward().rarity === 'uncommon' ? t("meta.rarity.uncommon") : t("meta.rarity.common")}
                            </Text>
                            <Text class={`text-fg max-w-full font-semibold`}>
                              {reward().name}
                            </Text>
                          </View>
                        </View>
                      </View>
                      )}
                    </For></Show>
                  </View>
                </Scroll>
              </Show>
          }>
            <View class={`grow shrink basis-0 items-center justify-center p-6 gap-3`} role="status">
              <Spinner name={(props.state ?? "sealed") === 'loading' ? t("meta.loading") : t("meta.reveal.revealing")} />
              <Text>{(props.state ?? "sealed") === 'loading' ? t("meta.loading") : t("meta.reveal.revealing")}</Text>
            </View>
          </Show>
      }>
        <ErrorState body={props.error ?? ''} />
      </Show>
      <Show when={!props.error}>
        <View class={`flex-row items-center w-full justify-center gap-1`}>
          <Show when={(props.state ?? "sealed") === 'sealed'} fallback={
              <Show when={(props.state ?? "sealed") === 'revealing' && (props.canSkip ?? true)} fallback={
                  <Show when={(props.state ?? "sealed") === 'revealed'}>
                    <View class={`text-accent shrink basis-[18.571rem] min-h-[50.6px] gap-1 compact:grow compact:shrink compact:basis-auto`} role="button" disabled={(props.disabled ?? false)} label={props.continueLabel || t("meta.continue")} onPress={(e) => props.onContinue?.(e)}>
                      <Text class="whitespace-normal">{props.continueLabel || t("meta.continue")}</Text>
                    </View>
                  </Show>
              }>
                <View class={`shrink basis-[18.571rem] min-h-[50.6px] gap-1 compact:grow compact:shrink compact:basis-auto`} role="button" disabled={(props.disabled ?? false)} label={t("meta.reveal.skip")} onPress={(e) => props.onSkip?.(e)}>
                  <Text class="whitespace-normal">{t("meta.reveal.skip")}</Text>
                </View>
              </Show>
          }>
            <View class={`text-accent shrink basis-[18.571rem] min-h-[50.6px] gap-1 compact:grow compact:shrink compact:basis-auto`} role="button" disabled={(props.disabled ?? false)} label={t("meta.reveal.open")} onPress={(e) => props.onOpen?.(e)}>
              <Text class="whitespace-normal">{t("meta.reveal.open")}</Text>
            </View>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RevealProps>> = {
  default: {"title":"Abbey chest","kicker":"Expedition rewards","body":"Open the chest to see what your party recovered.","icon":"bag","state":"sealed"},
  loading: {"state":"loading"},
  revealing: {"title":"Abbey chest","kicker":"Expedition rewards","state":"revealing","canSkip":true},
  revealed: {"title":"Abbey chest","kicker":"Expedition rewards · 4 found","state":"revealed","rewards":[{"id":"blade","name":"Ember Blade of the Ninefold Gate","icon":"sword","amount":"","rarity":"legendary","fresh":true},{"id":"band","name":"Moonlit Band","icon":"gem","amount":"","rarity":"rare","fresh":true},{"id":"draught","name":"Healing draught","icon":"potion","amount":"×3","rarity":"uncommon","fresh":false},{"id":"gold","name":"Gold","icon":"coin","amount":"×1,250","rarity":"common","fresh":false}]},
  empty: {"state":"revealed","rewards":[]},
  error: {"error":"Your rewards are saved. Reconnect to view them."},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"open":"Open the sealed rewards","skip":"Finish the reveal early","inspect":"Inspect a revealed reward by its key","continue":"Continue after the reveal","retry":"Retry the reveal"} as const;
