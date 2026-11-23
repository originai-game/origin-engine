import { For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatNumber, t } from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import { Spinner } from "../base/spinner";
import "./general.en.ogt";

export interface ShopProps {
  /** The shop's heading; empty takes the group's (Shop). */
  title: string;
  /** A small line over the title naming the place (Store, a merchant's town); empty hides it. */
  kicker?: string;
  /** The player's balances shown in the band, keyed rows of { id, icon, label, amount }: amount as the game formats it; label names the currency for a screen reader. */
  wallet: ReadonlyArray<{ id: string; icon: string; label: string; amount: string }>;
  /** The categories, keyed rows of { id, label, count }: count, how many offers the category holds, is shown beside its label above 0; empty shows no tab row. */
  tabs: ReadonlyArray<{ id: string; label: string; count: number }>;
  /** The category shown. */
  tab: string;
  /** The offers in the shown category, in the game's order: keyed rows of { id, name, icon, detail, rarity, badge, badgeTone, featured, soldOut, owned, affordable }: detail a second line (×500, 7 days); rarity common, uncommon, rare, epic, legendary or mythic, the light behind the art (empty is common); badge a corner tag the game words (−30%, New), empty none, on gold unless badgeTone is success; featured a wide tile; soldOut, owned and affordable the game's verdicts: a sold out, owned or unaffordable (affordable false) tile keeps the focus, is marked disabled, says why in words on its foot and reports nothing. */
  offers: ReadonlyArray<{ id: string; name: string; icon: string; detail: string; rarity: string; badge: string; badgeTone: string; featured: boolean; soldOut: boolean; owned: boolean; affordable: boolean }>;
  /** Every offer's price, one row per currency: keyed rows of { id, offer, icon, label, amount, was, short }: offer an offer's id; amount and was (the struck price before a discount, empty none) as the game formats them; label the currency's name; short marks the currency the player lacks. */
  prices: ReadonlyArray<{ id: string; offer: string; icon: string; label: string; amount: string; was: string; short: boolean }>;
  /** The offer the focus rests on; empty is none. */
  selected: string;
  /** The catalogue is on its way: a spinner stands in. */
  loading?: boolean;
  /** Why the catalogue could not load, as the game words it; empty is none. Shows a Try again button. */
  error: string;
  /** The buy event, with the row's key where it comes from a row. */
  onBuy?: (e: UiEvent, key?: string) => void;
  /** The choosetab event, with the row's key where it comes from a row. */
  onChoosetab?: (e: UiEvent, key?: string) => void;
  /** The retry event, with the row's key where it comes from a row. */
  onRetry?: (e: UiEvent, key?: string) => void;
  /** The tabNext event, with the row's key where it comes from a row. */
  onTabNext?: (e: UiEvent, key?: string) => void;
  /** The tabPrev event, with the row's key where it comes from a row. */
  onTabPrev?: (e: UiEvent, key?: string) => void;
}

/**
 * A storefront: a band with a kicker over the title and the player's
 * balances as chips, category tabs with their counts between the bumper
 * caps, and a grid of offer tiles. A tile is its art in a well over a glow
 * of its rarity, corner tags (Featured, and a badge the game words), its
 * name and detail, and one ruled foot with its price in one or more
 * currencies (a discount's old price struck before it, and named: 700
 * Gems, was 1,000). The game owns the catalogue, every price and every
 * verdict (sold out, owned, unaffordable); the shop reports buy (a tile's
 * key, which the game answers with a ShopConfirm), choosetab (a category's
 * key), tabNext and tabPrev (E/Q, bumpers) and retry. A tile the game
 * refused has no buy handler: it keeps the focus so a player can read it,
 * is marked disabled (aria-disabled), dims its art and says why in words
 * on its foot. The selected tile wears corner brackets. The grid fills the
 * height it is given and scrolls inside it; narrow, it is two columns and
 * the tabs scroll sideways.
 */
export function Shop(props: ShopProps) {
  return (
    <View class={`grow shrink basis-0 border-rim nine-panel-cut bg-glass-hi text-fg p-4 gap-2 border disabled:text-fg-subtle`} label={props.title !== '' ? props.title : t("general.shop.name")} busy={props.loading}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class={`flex-row flex-wrap items-end min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <View class="grow shrink basis-30 gap-0.5 compact:basis-full">
          <Show when={props.kicker}>
            <Text class={`text-xs font-semibold tracking-kicker uppercase text-fg-subtle whitespace-nowrap text-ellipsis overflow-hidden`}>
              {props.kicker}
            </Text>
          </Show>
          <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg line-clamp-2 m-0 overflow-hidden`}>
            {props.title !== '' ? props.title : t("general.shop.name")}
          </Text>
        </View>
        <Show when={props.wallet.length}>
          <View class={`flex-row flex-wrap items-center justify-end max-w-full p-0 m-0 gap-1 compact:justify-start shrink`} label={t("general.shop.wallet")}>
            <For each={props.wallet}>
              {(w) => (
              <View class={`flex-row items-center max-w-full pr-2 pl-1 bg-glass inset-ring-1 inset-ring-rim py-0.5 gap-1 rounded-capsule`} label={t("general.shop.amount", { amount: w().amount, currency: w().label })}>
                <Icon class="size-5" name={w().icon} />
                <Text class={`tabular-nums text-fg whitespace-nowrap font-bold text-ellipsis overflow-hidden shrink`}>
                  {w().amount}
                </Text>
              </View>
              )}
            </For>
          </View>
        </Show>
      </View>
      <Show when={props.tabs.length}>
        <View class="flex-row items-center w-full border-b border-rim-soft gap-1" role="tablist" label={t("general.shop.tabs")} onAction={{ tabNext: (e) => props.onTabNext?.(e), tabPrev: (e) => props.onTabPrev?.(e) }}>
          <Scroll class="grow shrink basis-0">
            <View class="flex-row items-center pr-12 gap-0">
              <For each={props.tabs}>
                {(c) => (
                <View class={`flex-row items-center justify-center group/tab text-fg-muted transition duration-317 min-h-11 min-w-11 py-0 px-2 gap-1 data-selected:text-fg`} role="tab" selected={c().id === props.tab} label={c().label} onPress={(e) => props.onChoosetab?.(e, c().id)} data-selected={c().id === props.tab}>
                  <Text class="whitespace-nowrap">{c().label}</Text>
                  <Show when={c().count > 0}>
                    <Text class={`text-xs font-bold text-fg-subtle py-0 px-1 rounded-capsule group-data-selected/tab:bg-none group-data-selected/tab:bg-hud-accent group-data-selected/tab:text-hud-accent-fg ${packClass("Chip", "frame")}`}>
                      {formatNumber(c().count)}
                    </Text>
                  </Show>
                  <Text class="absolute bottom-0 h-0.75 bg-hud-accent shadow-glow opacity-0 inset-x-1 group-data-selected/tab:opacity-100" />
                </View>
                )}
              </For>
            </View>
          </Scroll>
        </View>
      </Show>
      <Show when={props.error !== ''} fallback={
          <Show when={props.loading} fallback={
              <Show when={props.offers.length} fallback={
                  <View class="grow shrink basis-0 justify-center items-center w-full p-4 gap-2" role="status">
                    <Icon class="size-12" name="bag" />
                    <Text class="text-center m-0">{t("general.shop.empty")}</Text>
                  </View>
              }>
                <Scroll class="grow shrink basis-0 gap-2">
                  <View class="grid grid-cols-fill-45 w-full p-1 gap-1 compact:grid-cols-2" role="grid" label={props.title !== '' ? props.title : t("general.shop.name")}>
                    <For each={props.offers}>
                      {(o) => (
                      <Show when={!o().soldOut && !o().owned && o().affordable} fallback={
                          <View class={`group/offer w-full pt-1 pb-2 bg-tile nine-plate-cut px-1 hover:bg-none hover:bg-tile data-featured:col-span-2`} role="gridcell" data-sold-out={o().soldOut} data-owned={o().owned} data-affordable={o().affordable} selected={o().id === props.selected} disabled={true} label={t("general.shop.refused", { name: o().name, status: o().soldOut ? t("general.shop.soldOut") : o().owned ? t("general.shop.owned") : t("general.shop.short") })} data-plain={!o().featured} data-featured={o().featured} data-refused={true} data-selected={o().id === props.selected} data-tier-common={(o().rarity || 'common') === "common"}>
                            <View class="w-full gap-1">
                              <View class={`justify-center items-center w-full h-24 text-border-strong bg-well nine-plate-cut gap-2 compact:h-15`}>
                                <View class="absolute opacity-26 inset-0" />
                                <Icon class="size-12 compact:size-9" name={o().icon} />
                              </View>
                              <Show when={o().featured}>
                                <Text class={`absolute top-2 left-2 max-w-[calc(50%-12px)] bg-hud-accent text-hud-accent-fg text-xs font-bold tracking-label uppercase leading-tight whitespace-nowrap text-ellipsis nine-plate-cut py-0.5 px-1 overflow-hidden`}>
                                  {t("general.shop.featured")}
                                </Text>
                              </Show>
                              <Show when={o().badge}>
                                <Text class={`font-bold text-xs leading-tight tabular-nums bg-gold text-gold-fg transition duration-317 absolute top-2 tracking-label uppercase whitespace-nowrap text-ellipsis right-2 py-0.5 px-1 overflow-hidden data-tone-success:bg-none data-tone-success:bg-success data-tone-success:text-success-fg`} data-tone-success={(o().badgeTone) === 'success'}>
                                  {o().badge}
                                </Text>
                              </Show>
                              <View class="w-full gap-0.5">
                                <Text class={`text-fg text-start text-base font-bold leading-tight m-0 group-data-refused/offer:text-fg-muted`}>
                                  {o().name}
                                </Text>
                                <Show when={o().detail}>
                                  <Text class={`text-caption text-fg-muted text-start whitespace-nowrap text-ellipsis overflow-hidden`}>
                                    {o().detail}
                                  </Text>
                                </Show>
                              </View>
                              <View class={`flex-row flex-wrap items-center w-full mt-auto pt-1 border-t border-rim gap-1`} label={t("general.shop.price")}>
                                <For each={props.prices}>
                                  {(p) => (
                                  <Show when={p().offer === o().id}>
                                    <View class={`flex-row items-center max-w-full text-lg text-fg gap-0.5 data-short:text-danger`} data-short={p().short}>
                                      <Icon class="size-4.5" name={p().icon} />
                                      <Show when={p().was}>
                                        <Text class={`line-through text-fg-muted mr-1 whitespace-nowrap text-reading font-semibold shrink`}>
                                          {p().was}
                                        </Text>
                                      </Show>
                                      <Text class={`tabular-nums text-fg whitespace-nowrap font-bold text-ellipsis overflow-hidden shrink`} label={p().was ? t("general.shop.amountWas", { amount: p().amount, currency: p().label, was: p().was }) : t("general.shop.amount", { amount: p().amount, currency: p().label })}>
                                        {p().amount}
                                      </Text>
                                    </View>
                                  </Show>
                                  )}
                                </For>
                                <Show when={o().soldOut}>
                                  <View class={`flex-row items-center ml-auto text-xs font-bold tracking-label uppercase text-fg-muted nine-plate-cut py-0.5 px-1 gap-0.5 ${packClass("Chip", "frame")}`}>
                                    <Text class="whitespace-nowrap shrink">{t("general.shop.soldOut")}</Text>
                                  </View>
                                </Show>
                                <Show when={o().owned && !o().soldOut}>
                                  <View class={`flex-row items-center ml-auto text-xs font-bold tracking-label uppercase text-fg-muted nine-plate-cut py-0.5 px-1 gap-0.5 data-kind-owned:bg-none data-kind-owned:bg-success data-kind-owned:text-success-fg ${packClass("Chip", "frame")}`} data-kind-owned={true}>
                                    <Icon class="size-3" name="check" />
                                    <Text class="whitespace-nowrap shrink">{t("general.shop.owned")}</Text>
                                  </View>
                                </Show>
                                <Show when={!o().affordable && !o().soldOut && !o().owned}>
                                  <View class={`flex-row items-center ml-auto text-xs font-bold tracking-label uppercase text-fg-muted nine-plate-cut py-0.5 px-1 gap-0.5 data-kind-short:bg-none data-kind-short:bg-danger data-kind-short:text-danger-fg ${packClass("Chip", "frame")}`} data-kind-short={true}>
                                    <Text class="whitespace-nowrap shrink">{t("general.shop.short")}</Text>
                                  </View>
                                </Show>
                              </View>
                            </View>
                            <Show when={o().id === props.selected}>
                              <View class={`absolute pointer-events-none inset-0`} />
                            </Show>
                          </View>
                      }>
                        <View class={`group/offer w-full pt-1 pb-2 bg-tile nine-plate-cut px-1 hover:bg-none hover:bg-tile data-featured:col-span-2`} role="gridcell" data-sold-out={o().soldOut} data-owned={o().owned} data-affordable={o().affordable} selected={o().id === props.selected} label={o().name} onPress={(e) => props.onBuy?.(e, o().id)} data-plain={!o().featured} data-featured={o().featured} data-selected={o().id === props.selected} data-tier-common={(o().rarity || 'common') === "common"}>
                          <View class="w-full gap-1">
                            <View class={`justify-center items-center w-full h-24 text-border-strong bg-well nine-plate-cut gap-2 compact:h-15`}>
                              <View class="absolute opacity-26 inset-0" />
                              <Icon class="size-12 compact:size-9" name={o().icon} />
                            </View>
                            <Show when={o().featured}>
                              <Text class={`absolute top-2 left-2 max-w-[calc(50%-12px)] bg-hud-accent text-hud-accent-fg text-xs font-bold tracking-label uppercase leading-tight whitespace-nowrap text-ellipsis nine-plate-cut py-0.5 px-1 overflow-hidden`}>
                                {t("general.shop.featured")}
                              </Text>
                            </Show>
                            <Show when={o().badge}>
                              <Text class={`font-bold text-xs leading-tight tabular-nums bg-gold text-gold-fg transition duration-317 absolute top-2 tracking-label uppercase whitespace-nowrap text-ellipsis right-2 py-0.5 px-1 overflow-hidden data-tone-success:bg-none data-tone-success:bg-success data-tone-success:text-success-fg`} data-tone-success={(o().badgeTone) === 'success'}>
                                {o().badge}
                              </Text>
                            </Show>
                            <View class="w-full gap-0.5">
                              <Text class={`text-fg text-start text-base font-bold leading-tight m-0`}>
                                {o().name}
                              </Text>
                              <Show when={o().detail}>
                                <Text class={`text-caption text-fg-muted text-start whitespace-nowrap text-ellipsis overflow-hidden`}>
                                  {o().detail}
                                </Text>
                              </Show>
                            </View>
                            <View class={`flex-row flex-wrap items-center w-full mt-auto pt-1 border-t border-rim gap-1`} label={t("general.shop.price")}>
                              <For each={props.prices}>
                                {(p) => (
                                <Show when={p().offer === o().id}>
                                  <View class={`flex-row items-center max-w-full text-lg text-fg gap-0.5 data-short:text-danger`} data-short={p().short}>
                                    <Icon class="size-4.5" name={p().icon} />
                                    <Show when={p().was}>
                                      <Text class={`line-through text-fg-muted mr-1 whitespace-nowrap text-reading font-semibold shrink`}>
                                        {p().was}
                                      </Text>
                                    </Show>
                                    <Text class={`tabular-nums text-fg whitespace-nowrap font-bold text-ellipsis overflow-hidden shrink`} label={p().was ? t("general.shop.amountWas", { amount: p().amount, currency: p().label, was: p().was }) : t("general.shop.amount", { amount: p().amount, currency: p().label })}>
                                      {p().amount}
                                    </Text>
                                  </View>
                                </Show>
                                )}
                              </For>
                            </View>
                          </View>
                          <Show when={o().id === props.selected}>
                            <View class={`absolute pointer-events-none inset-0`} />
                          </Show>
                        </View>
                      </Show>
                      )}
                    </For>
                  </View>
                </Scroll>
              </Show>
          }>
            <View class="grow shrink basis-0 justify-center items-center w-full p-4 gap-2" role="status">
              <Spinner name={t("general.shop.loading")} />
              <Text class="text-center m-0">{t("general.shop.loading")}</Text>
            </View>
          </Show>
      }>
        <View class="grow shrink basis-0 justify-center items-center w-full p-4 gap-2" role="alert">
          <Icon class="size-12" name="close" />
          <Text class="text-center m-0">{props.error}</Text>
          <Button class="min-h-11" onPress={(e) => props.onRetry?.(e)}><Text>{t("general.shop.retry")}</Text></Button>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShopProps>> = {
  default: {"title":"","kicker":"Store","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"12,450"},{"id":"gems","icon":"gem","label":"Gems","amount":"3,200"}],"tabs":[{"id":"featured","label":"Featured","count":3},{"id":"items","label":"Items","count":12},{"id":"looks","label":"Looks","count":8},{"id":"bundles","label":"Bundles","count":2}],"tab":"featured","offers":[{"id":"bundle","name":"Founder Bundle","icon":"crown","detail":"3 items, 500 gems","rarity":"legendary","badge":"−30%","badgeTone":"","featured":true,"soldOut":false,"owned":false,"affordable":true},{"id":"potion","name":"Potion Pack","icon":"potion","detail":"×10","rarity":"uncommon","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"cape","name":"Ember Cape","icon":"fire","detail":"Look","rarity":"epic","badge":"New","badgeTone":"success","featured":false,"soldOut":false,"owned":true,"affordable":true},{"id":"key","name":"Vault Key","icon":"key","detail":"×1","rarity":"rare","badge":"","badgeTone":"","featured":false,"soldOut":true,"owned":false,"affordable":true},{"id":"shield","name":"Dragonscale Shield","icon":"shield","detail":"Shield","rarity":"legendary","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":false},{"id":"map","name":"Map of the Reaches","icon":"map","detail":"Reveals the east","rarity":"common","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":true},{"id":"band","name":"Moonlit Band","icon":"gem","detail":"Ring · +4 Intellect","rarity":"rare","badge":"","badgeTone":"","featured":false,"soldOut":false,"owned":false,"affordable":true}],"prices":[{"id":"bundle-gems","offer":"bundle","icon":"gem","label":"Gems","amount":"700","was":"1,000","short":false},{"id":"potion-gold","offer":"potion","icon":"coin","label":"Gold","amount":"1,200","was":"","short":false},{"id":"potion-gems","offer":"potion","icon":"gem","label":"Gems","amount":"40","was":"","short":false},{"id":"cape-gems","offer":"cape","icon":"gem","label":"Gems","amount":"250","was":"","short":false},{"id":"key-gold","offer":"key","icon":"coin","label":"Gold","amount":"5,000","was":"","short":false},{"id":"shield-gems","offer":"shield","icon":"gem","label":"Gems","amount":"4,800","was":"","short":true},{"id":"map-gold","offer":"map","icon":"coin","label":"Gold","amount":"800","was":"","short":false},{"id":"band-gold","offer":"band","icon":"coin","label":"Gold","amount":"3,250","was":"","short":false}],"selected":"bundle","loading":false,"error":""},
  empty: {"title":"","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"0"}],"tabs":[{"id":"items","label":"Items","count":0}],"tab":"items","offers":[],"prices":[],"selected":"","loading":false,"error":"","kicker":""},
  loading: {"title":"","wallet":[],"tabs":[],"tab":"","offers":[],"prices":[],"selected":"","loading":true,"error":"","kicker":""},
  error: {"title":"","wallet":[],"tabs":[],"tab":"","offers":[],"prices":[],"selected":"","loading":false,"error":"The shop could not be reached.","kicker":""},
  overflow: {"title":"The Wandering Merchant of the Far Eastern Reaches","wallet":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,234,567,890"},{"id":"gems","icon":"gem","label":"Gems","amount":"99,999"},{"id":"tokens","icon":"star","label":"Tokens","amount":"12"}],"tabs":[{"id":"a","label":"Featured","count":0},{"id":"b","label":"Consumables","count":0},{"id":"c","label":"Equipment","count":0},{"id":"d","label":"Cosmetics","count":0},{"id":"e","label":"Bundles","count":0},{"id":"f","label":"Seasonal","count":0}],"tab":"b","offers":[{"id":"o1","name":"An Extraordinarily Long Offer Name That Must Wrap","icon":"potion","detail":"x10 with a long detail line","badge":"Limited time","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"rare","badgeTone":""},{"id":"o2","name":"Potion","icon":"potion","detail":"x1","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":false,"rarity":"common","badgeTone":""},{"id":"o3","name":"Elixir","icon":"drop","detail":"x5","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"uncommon","badgeTone":""},{"id":"o4","name":"Bomb","icon":"bomb","detail":"x3","badge":"","featured":false,"soldOut":true,"owned":false,"affordable":true,"rarity":"epic","badgeTone":""},{"id":"o5","name":"Apple","icon":"apple","detail":"x20","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"rare","badgeTone":""},{"id":"o6","name":"Leaf","icon":"leaf","detail":"x8","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"common","badgeTone":""},{"id":"o7","name":"Bolt","icon":"bolt","detail":"x2","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"uncommon","badgeTone":""},{"id":"o8","name":"Shield","icon":"shield","detail":"x1","badge":"","featured":false,"soldOut":false,"owned":false,"affordable":true,"rarity":"epic","badgeTone":""}],"prices":[{"id":"p1a","offer":"o1","icon":"coin","label":"Gold","amount":"123,456,789","was":"200,000,000","short":false},{"id":"p1b","offer":"o1","icon":"gem","label":"Gems","amount":"9,999","was":"","short":false},{"id":"p1c","offer":"o1","icon":"star","label":"Tokens","amount":"12","was":"","short":false},{"id":"p2","offer":"o2","icon":"star","label":"Tokens","amount":"99","was":"","short":true},{"id":"p3","offer":"o3","icon":"coin","label":"Gold","amount":"800","was":"","short":false},{"id":"p4","offer":"o4","icon":"coin","label":"Gold","amount":"300","was":"","short":false},{"id":"p5","offer":"o5","icon":"coin","label":"Gold","amount":"40","was":"","short":false},{"id":"p6","offer":"o6","icon":"coin","label":"Gold","amount":"90","was":"","short":false},{"id":"p7","offer":"o7","icon":"gem","label":"Gems","amount":"15","was":"","short":false},{"id":"p8","offer":"o8","icon":"coin","label":"Gold","amount":"2,000","was":"","short":false}],"selected":"o3","loading":false,"error":"","kicker":"The Far Eastern Reaches Trading Company"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"tabPrev":"Previous category","tabNext":"Next category"} as const;
