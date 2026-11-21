import { For, Icon, Scroll, Show, Text, type UiEvent, View, formatNumber, t } from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./idle.en.ogt";

export interface IdlePurchasesProps {
  /** The list's heading; empty takes the group's (Upgrades). */
  title: string;
  /** The buy amounts offered, keyed rows of { id, label }: label the amount as the game words it (it wins; empty reads the key max as Max and any other key as ×key: ×1, ×10, ×100). An empty list hides the choice. */
  amounts?: ReadonlyArray<{ id: string; label: string }>;
  /** The buy amount chosen, one of the amounts' keys. */
  amount: string;
  /** The purchases in the game's order, keyed rows of { id, icon, name, effect, level, count, cost, costIcon, affordable, maxed, locked }: effect what it does as the game words it; level its level (0 hides it); count how many the chosen amount buys, as the game words it (×37; empty hides it); cost the price of that many, formatted; costIcon the currency's icon; affordable the player can pay it (else named Not enough); maxed nothing more to buy (named Maxed); locked not yet available (named Locked). Buy is offered only when affordable and neither maxed nor locked. */
  upgrades?: ReadonlyArray<{ id: string; icon: string; name: string; effect: string; level: number; count: string; cost: string; costIcon: string; affordable: boolean; maxed: boolean; locked: boolean }>;
  /** The buy event, with the row's key where it comes from a row. */
  onBuy?: (e: UiEvent, key?: string) => void;
  /** The chooseamount event, with the row's key where it comes from a row. */
  onChooseamount?: (e: UiEvent, key?: string) => void;
}

/**
 * The upgrade shop of an idle game: a ×1 / ×10 / ×100 / Max choice of how
 * many to buy (chooseamount, the amount's key), then one row per upgrade
 * with its icon, name, effect, level, how many the chosen amount buys and
 * what that costs, and Buy (buy, the upgrade's key). The game prices every
 * amount and decides what can be bought: Buy is enabled only when the row
 * is affordable and neither maxed nor locked, and each refused state is a
 * word (Not enough, Maxed, Locked). It fills its height and scrolls.
 */
export function IdlePurchases(props: IdlePurchasesProps) {
  return (
    <View class={`grow shrink basis-0 border-rim rounded-tl-sheet rounded-tr-none rounded-br-sheet rounded-bl-none bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg w-full p-4 gap-3 border disabled:text-fg-subtle`} label={props.title !== '' ? props.title : t("idle.purchases.name")}>
      <View class="absolute left-4 right-[30%] top-0 h-px bg-linear-to-r from-slot-selected-edge to-transparent to-70% pointer-events-none" />
      <View class={`flex-row flex-wrap items-end min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <Text class={`grow shrink basis-0 font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg whitespace-nowrap text-ellipsis m-0 overflow-hidden`}>
          {props.title !== '' ? props.title : t("idle.purchases.name")}
        </Text>
        <Show when={(props.amounts ?? []).length}>
          <View class="flex-row items-center bg-[#04060c8c] shadow-[inset_0px_0px_0px_1px_#4e586c] p-0.5 gap-0.5 rounded-control compact:grow compact:shrink-0 compact:basis-full" role="radiogroup" label={t("idle.purchases.amount")}>
            <For each={props.amounts}>
              {(a) => (
              <View class={`min-h-11 min-w-11 text-fg-muted py-0 px-2 rounded-control compact:min-w-0 compact:grow compact:shrink compact:basis-0 data-checked:bg-material-edge data-checked:bg-none data-checked:shadow-[inset_0px_1px_0px_#ffffff33] data-checked:text-fg`} role="radio" data-amount={a().id} onPress={(e) => props.onChooseamount?.(e, a().id)} data-checked={a().id === props.amount}>
                <Text>{a().label ? a().label : a().id === 'max' ? t("idle.purchases.max") : t("idle.purchases.times", { count: a().id })}</Text>
              </View>
              )}
            </For>
          </View>
        </Show>
      </View>
      <Show when={(props.upgrades ?? []).length} fallback={
          <Text class={`text-caption text-fg-muted grow shrink basis-0 text-center p-4 m-0`} role="status">
            {t("idle.purchases.empty")}
          </Text>
      }>
        <View class="grow shrink basis-0 gap-0">
          <Scroll class="grow shrink basis-0 gap-0 ">
            <View class="w-full pt-0 pb-4 px-0 m-0 gap-1" label={props.title !== '' ? props.title : t("idle.purchases.name")}>
              <For each={props.upgrades}>
                {(u) => (
                <View class={`group/row flex-row items-center rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-[#090e1a8c] text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#343d4f] border-l-3 border-[#38e8ff] py-1 px-2 gap-3 data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45 compact:flex-wrap`} label={u().name} data-maxed={u().maxed} data-not-affordable={!(u().affordable)} data-locked={u().locked}>
                  <View class={`justify-center items-center bg-linear-to-b from-shade to-transparent to-55% bg-tile shadow-[inset_0px_0px_0px_1.5px_#4b5568,inset_0px_1px_3px_0px_#00000099,0px_1px_0px_0px_#ffffff0a] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none gap-2 size-[3.429rem] group-data-locked/row:opacity-45 compact:size-[2.857rem]`}>
                    <Icon class="size-[64%]" name={u().locked ? 'lock' : u().icon} />
                  </View>
                  <View class="grow shrink basis-0 gap-0.5 compact:grow compact:shrink compact:basis-0 group-data-locked/row:opacity-45">
                    <View class="flex-row items-center w-full gap-2">
                      <Text class={`text-fg shrink text-start whitespace-nowrap text-ellipsis text-[1.214rem] font-semibold overflow-hidden`}>
                        {u().name}
                      </Text>
                      <Show when={u().level > 0}>
                        <Text class={`font-bold text-[0.857rem] leading-4.5 tabular-nums bg-material-edge text-fg-muted transition duration-317 whitespace-nowrap py-0 px-1 rounded-capsule`} data-tag-level={true}>
                          {t("idle.purchases.level", { level: formatNumber(u().level) })}
                        </Text>
                      </Show>
                    </View>
                    <Text class={`text-caption text-fg-muted text-start m-0`}>
                      {u().effect}
                    </Text>
                  </View>
                  <View class="items-end max-w-1/2 gap-1 compact:flex-row compact:grow compact:shrink-0 compact:basis-full compact:items-center compact:max-w-full compact:justify-end">
                    <Show when={u().maxed}>
                      <Text class={`font-bold text-[0.857rem] leading-4.5 tabular-nums rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-material-edge text-fg-muted transition duration-317 whitespace-nowrap tracking-[0.14em] uppercase py-0 px-1`}>
                        {t("idle.purchases.maxed")}
                      </Text>
                    </Show>
                    <Show when={!u().maxed && u().locked}>
                      <Text class={`font-bold text-[0.857rem] leading-4.5 tabular-nums rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-material-edge text-fg-muted transition duration-317 whitespace-nowrap tracking-[0.14em] uppercase py-0 px-1`}>
                        {t("idle.purchases.locked")}
                      </Text>
                    </Show>
                    <Show when={!u().maxed && !u().locked && !u().affordable}>
                      <Text class={`font-bold text-[0.857rem] leading-4.5 tabular-nums rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none bg-material-edge text-fg-muted transition duration-317 whitespace-nowrap tracking-[0.14em] uppercase py-0 px-1 data-tag-short:bg-none data-tag-short:bg-danger/12 data-tag-short:text-danger`} data-tag-short={true}>
                        {t("idle.purchases.short")}
                      </Text>
                    </Show>
                    <Show when={!u().maxed}>
                      <Button class="text-accent min-h-11 min-w-32 max-w-full py-0.5 px-2 data-not-affordable:text-danger" disabled={!u().affordable || u().maxed || u().locked} label={u().cost ? t("idle.purchases.buyFor", { name: u().name, cost: u().cost }) : t("idle.purchases.buyNamed", { name: u().name })} onPress={(e) => props.onBuy?.(e, u().id)} data-not-affordable={!(u().affordable)}>
                        <View class="items-center max-w-full gap-0">
                          <Show when={u().count}>
                            <Text class={`tabular-nums text-current whitespace-nowrap text-[0.857rem] font-bold opacity-80`}>
                              {u().count}
                            </Text>
                          </Show>
                          <Show when={u().cost} fallback={
                              <Text>{t("idle.purchases.buy")}</Text>
                          }>
                            <View class={`flex-row items-center text-current max-w-full font-bold gap-0.5`}>
                              <Icon class="size-4" name={u().costIcon} />
                              <Text class="whitespace-nowrap text-ellipsis tabular-nums overflow-hidden shrink">
                                {u().cost}
                              </Text>
                            </View>
                          </Show>
                        </View>
                      </Button>
                    </Show>
                  </View>
                </View>
                )}
              </For>
            </View>
          </Scroll>
          <View class={`absolute bottom-0 h-4 pointer-events-none bg-linear-to-b from-transparent to-surface inset-x-0`} />
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<IdlePurchasesProps>> = {
  default: {"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"1","upgrades":[{"id":"pick","icon":"sword","name":"Sharper Picks","effect":"Mines yield +10%","level":4,"count":"×1","cost":"1.2K","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"cart","icon":"bag","name":"Bigger Carts","effect":"Wood store +50%","level":1,"count":"×1","cost":"48K","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"crown","icon":"crown","name":"Royal Charter","effect":"All income ×2","level":5,"count":"","cost":"","costIcon":"gem","affordable":false,"maxed":true,"locked":false},{"id":"star","icon":"star","name":"Starlight","effect":"Unlocks at level 50","level":0,"count":"","cost":"1M","costIcon":"gem","affordable":false,"maxed":false,"locked":true}]},
  max: {"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"max","upgrades":[{"id":"pick","icon":"sword","name":"Sharper Picks","effect":"Mines yield +10%","level":4,"count":"×37","cost":"412K","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"cart","icon":"bag","name":"Bigger Carts","effect":"Wood store +50%","level":1,"count":"×0","cost":"48K","costIcon":"coin","affordable":false,"maxed":false,"locked":false}]},
  disabled: {"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"100","upgrades":[{"id":"pick","icon":"sword","name":"Sharper Picks","effect":"Mines yield +10%","level":4,"count":"×100","cost":"9.8M","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"star","icon":"star","name":"Starlight","effect":"Unlocks at level 50","level":0,"count":"","cost":"1M","costIcon":"gem","affordable":false,"maxed":false,"locked":true}]},
  maxed: {"title":"","amounts":[],"amount":"","upgrades":[{"id":"crown","icon":"crown","name":"Royal Charter","effect":"All income ×2","level":5,"count":"","cost":"","costIcon":"gem","affordable":false,"maxed":true,"locked":false},{"id":"key","icon":"key","name":"Vault Key","effect":"Opens the vault","level":1,"count":"","cost":"","costIcon":"coin","affordable":true,"maxed":true,"locked":false}]},
  empty: {"title":"","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"1","upgrades":[]},
  overflow: {"title":"Research and Development of the Grand Arcane Academy","amounts":[{"id":"1","label":""},{"id":"10","label":""},{"id":"100","label":""},{"id":"max","label":""}],"amount":"10","upgrades":[{"id":"u1","icon":"gear","name":"Self-Replicating Clockwork Automaton Assembly Lines","effect":"Every generator in every realm yields an additional four hundred percent","level":1234,"count":"×10","cost":"123.45 quattuordecillion","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"u2","icon":"sword","name":"Picks","effect":"+10%","level":4,"count":"×10","cost":"12K","costIcon":"coin","affordable":true,"maxed":false,"locked":false},{"id":"u3","icon":"bag","name":"Carts","effect":"+50%","level":1,"count":"×10","cost":"480K","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"u4","icon":"potion","name":"Elixirs","effect":"+5% speed","level":9,"count":"×10","cost":"3.3M","costIcon":"coin","affordable":false,"maxed":false,"locked":false},{"id":"u5","icon":"crown","name":"Charter","effect":"×2","level":5,"count":"","cost":"","costIcon":"gem","affordable":false,"maxed":true,"locked":false},{"id":"u6","icon":"star","name":"Starlight","effect":"Level 50","level":0,"count":"","cost":"1M","costIcon":"gem","affordable":false,"maxed":false,"locked":true},{"id":"u7","icon":"bolt","name":"Dynamos","effect":"+1 per tap","level":2,"count":"×10","cost":"77K","costIcon":"coin","affordable":true,"maxed":false,"locked":false}]},
};
