import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./general.en.ogt";

export interface ShopConfirmProps {
  /** The panel's heading, a small line over the offer's name; empty takes the group's (Confirm purchase). */
  title: string;
  /** The offer's name. */
  name: string;
  /** The offer's icon in the style's or the game's atlas. */
  icon: string;
  /** common, uncommon, rare, epic, legendary or mythic: the socket the icon sits in; empty is common. */
  rarity?: string;
  /** What the offer gives, as the game words it; empty hides it. */
  detail: string;
  /** What it costs, one group of rows per currency: keyed rows of { id, icon, label, amount, was, after, short, missing }: amount, was (struck, empty none) and after (the balance before and after, 3,200 → 2,500; empty hides it) as the game formats them; label the currency's name; short marks a currency the player lacks, and missing how much more it takes (1,600), shown in place of the balance. */
  prices: ReadonlyArray<{ id: string; icon: string; label: string; amount: string; was: string; after: string; short: boolean; missing: string }>;
  /** The game allows the purchase; false disables Buy. */
  canBuy?: boolean;
  /** Why the purchase is refused when no price says it, as the game words it (Requires level 40); shown over the buttons while canBuy is false. */
  reason?: string;
  /** The purchase is in flight: Buy is disabled and says it is working. */
  busy?: boolean;
  /** Why the last attempt failed, as the game words it; empty is none. */
  error: string;
  /** The cancel event, with the row's key where it comes from a row. */
  onCancel?: (e: UiEvent, key?: string) => void;
  /** The confirm event, with the row's key where it comes from a row. */
  onConfirm?: (e: UiEvent, key?: string) => void;
}

/**
 * The purchase confirmation the game shows after a shop's buy: a small
 * heading over the offer's name, its art in a socket of its rarity beside
 * its detail, and a ledger with one group per currency: the price (a
 * discount's old price struck before it, and named: 700 Gems, was 1,000),
 * then the balance before and after, or, for a currency the player lacks,
 * Not enough and how much more it takes. The game decides whether it may
 * be bought and what it costs; the panel reports confirm (Buy) and cancel
 * (Cancel, or Escape and pad B anywhere in it). Enter and pad A press the
 * focused button, Cancel first, so no key buys by accident. A refused
 * purchase disables Buy and says why (the short currency's row, or the
 * game's reason over the buttons); a failed one shows the game's error on
 * a strip and leaves Buy to try again. While busy Buy is disabled. The
 * ledger scrolls when it must; the buttons stay at the foot. It is
 * content, not a dialog: the host that shows it (the built-in confirm
 * screen, a sheet at the foot of a phone, or the game's own modal) owns
 * modality and focus.
 */
export function ShopConfirm(props: ShopConfirmProps) {
  return (
    <View class={`grow shrink basis-0 border-rim nine-panel-cut bg-glass-hi text-fg w-full max-w-105 max-h-full p-4 gap-2 border compact:nine-panel-cut disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title !== '' ? props.title : t("general.confirm.name")} busy={props.busy} onAction={{ back: (e) => props.onCancel?.(e) }} data-tier-common={(props.rarity || 'common') === "common"}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class={`min-h-12 border-b border-rim-soft w-full items-start text-start py-1 px-4 gap-0.5`}>
        <Text class={`font-display text-xs font-semibold leading-tight tracking-kicker uppercase text-fg-subtle m-0`}>
          {props.title !== '' ? props.title : t("general.confirm.name")}
        </Text>
        <Text class={`text-fg text-xl font-bold line-clamp-3 m-0 overflow-hidden`}>
          {props.name}
        </Text>
      </View>
      <Scroll class="shrink w-full gap-3">
        <View class="flex-row items-center w-full gap-3">
          <View class={`justify-center items-center bg-tile border-2 border-border-strong nine-plate-cut gap-2 size-24 compact:size-16 ${packClass("Slot", "frame")}`}>
            <Icon class="size-37/50" name={props.icon} />
          </View>
          <Show when={props.detail}>
            <Text class={`grow shrink basis-0 text-fg-muted m-0`}>
              {props.detail}
            </Text>
          </Show>
        </View>
        <View class={`w-full bg-well inset-ring-1 inset-ring-rim-soft py-1 px-2 m-0`} label={t("general.confirm.cost")}>
          <For each={props.prices}>
            {(p, i) => (
            <View class="group/entry w-full py-1 gap-1 data-later:border-t data-later:border-material-control-hover" data-later={i() >= 1} data-short={p().short}>
              <View class="flex-row items-center gap-y-0.5 gap-x-3 w-full flex-wrap">
                <Text class={`grow shrink basis-0 font-semibold text-xs leading-4.5 text-fg-subtle tracking-kicker uppercase`}>
                  {t("general.shop.price")}
                </Text>
                <View class={`flex-row items-center max-w-full ml-auto text-2xl text-fg gap-0.5 group-data-short/entry:text-danger`}>
                  <Icon class="size-5" name={p().icon} />
                  <Show when={p().was}>
                    <Text class={`line-through text-fg-muted mr-1 whitespace-nowrap text-reading font-semibold shrink`}>
                      {p().was}
                    </Text>
                  </Show>
                  <Text class={`tabular-nums text-fg font-bold whitespace-nowrap shrink`} label={p().was ? t("general.shop.amountWas", { amount: p().amount, currency: p().label, was: p().was }) : t("general.shop.amount", { amount: p().amount, currency: p().label })}>
                    {p().amount}
                  </Text>
                </View>
              </View>
              <Show when={p().short} fallback={
                  <Show when={p().after}>
                    <View class={`flex-row items-center gap-y-0.5 gap-x-3 w-full flex-wrap`} label={t("general.confirm.afterSaid", { currency: p().label, after: p().after })}>
                      <Text class={`grow shrink basis-0 font-semibold text-xs leading-4.5 text-fg-subtle tracking-kicker uppercase`}>
                        {t("general.confirm.balance")}
                      </Text>
                      <Text class="font-bold tabular-nums max-w-full ml-auto text-end">{p().after}</Text>
                    </View>
                  </Show>
              }>
                <View class={`flex-row items-center gap-y-0.5 gap-x-3 w-full flex-wrap`} label={p().missing ? t("general.confirm.missingSaid", { amount: p().missing, currency: p().label }) : t("general.confirm.shortOf", { currency: p().label })}>
                  <Text class="grow shrink basis-0 text-xs font-semibold tracking-kicker uppercase text-danger">
                    {t("general.confirm.shortOf", { currency: p().label })}
                  </Text>
                  <Show when={p().missing}>
                    <Text class="font-bold tabular-nums max-w-full ml-auto text-end text-danger">
                      {t("general.confirm.missing", { amount: p().missing })}
                    </Text>
                  </Show>
                </View>
              </Show>
            </View>
            )}
          </For>
        </View>
      </Scroll>
      <Show when={props.error}>
        <Text class={`w-full bg-danger text-danger-fg inset-shadow-bar inset-shadow-danger py-1 px-2 m-0`} role="alert">
          {props.error}
        </Text>
      </Show>
      <View class={`w-full gap-1`}>
        <Show when={!props.canBuy && !props.busy && props.reason}>
          <Text class={`text-danger m-0`} role="status">{props.reason}</Text>
        </Show>
        <View class="flex-row items-center w-full gap-1">
          <Button class={`grow shrink basis-0 min-h-12.75 gap-1 ${packClass("Button", "plate")}`} label={t("general.confirm.cancel")} onPress={(e) => props.onCancel?.(e)}>
            <Text>{t("general.confirm.cancel")}</Text>
          </Button>
          <Button class={`text-accent grow shrink basis-0 min-h-12.75 gap-1 compact:grow ${packClass("Button", "plate", "primary")}`} disabled={!props.canBuy || props.busy} label={props.busy ? t("general.confirm.busy") : t("general.confirm.buy")} onPress={(e) => props.onConfirm?.(e)}>
            <Text>{props.busy ? t("general.confirm.busy") : t("general.confirm.buy")}</Text>
          </Button>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShopConfirmProps>> = {
  default: {"title":"","name":"Founder Bundle","icon":"crown","rarity":"legendary","detail":"3 items and 500 gems","prices":[{"id":"gems","icon":"gem","label":"Gems","amount":"700","was":"1,000","after":"3,200 → 2,500","short":false,"missing":""}],"canBuy":true,"reason":"","busy":false,"error":""},
  disabled: {"title":"","name":"Dragonscale Shield","icon":"shield","rarity":"legendary","detail":"A shield for the long roads","prices":[{"id":"gems","icon":"gem","label":"Gems","amount":"4,800","was":"","after":"","short":true,"missing":"1,600"},{"id":"gold","icon":"coin","label":"Gold","amount":"10,000","was":"","after":"12,450 → 2,450","short":false,"missing":""}],"canBuy":false,"reason":"","busy":false,"error":""},
  refused: {"title":"","name":"Vault Key","icon":"key","rarity":"rare","detail":"×1","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"5,000","was":"","after":"12,450 → 7,450","short":false,"missing":""}],"canBuy":false,"reason":"Requires level 40","busy":false,"error":""},
  loading: {"title":"","name":"Potion Pack","icon":"potion","rarity":"uncommon","detail":"×10","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,200","was":"","after":"12,450 → 11,250","short":false,"missing":""}],"canBuy":true,"reason":"","busy":true,"error":""},
  error: {"title":"","name":"Potion Pack","icon":"potion","rarity":"uncommon","detail":"×10","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"1,200","was":"","after":"12,450 → 11,250","short":false,"missing":""},{"id":"gems","icon":"gem","label":"Gems","amount":"40","was":"","after":"3,200 → 3,160","short":false,"missing":""}],"canBuy":true,"reason":"","busy":false,"error":"The purchase did not go through. Nothing was charged."},
  overflow: {"title":"Confirm your purchase from the Wandering Merchant","name":"The Founder Bundle of the Far Eastern Reaches, Collector Edition","icon":"crown","rarity":"mythic","detail":"Three legendary items, five hundred gems and a title that everyone on the server will see","prices":[{"id":"gold","icon":"coin","label":"Gold","amount":"123,456,789","was":"200,000,000","after":"1,234,567,890 → 1,111,111,101","short":false,"missing":""},{"id":"tokens","icon":"star","label":"Tokens of the Eastern Treasury","amount":"99","was":"","after":"","short":true,"missing":"87"}],"canBuy":false,"reason":"","busy":false,"error":""},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Cancel"} as const;
