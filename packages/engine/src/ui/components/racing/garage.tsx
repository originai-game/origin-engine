import { For, Icon, Image, Scroll, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import { StatBar } from "../hud/stat-bar";
import "./racing.en.ogt";

export interface RacingGarageProps {
  /** The page's heading; empty takes the kit's (Garage). */
  title?: string;
  /** The cars in the game's order, keyed rows of { id, title, body, image, owned, locked, affordable, price }: title the car's name, body its class line, image a texture (og.load.texture, 0 none); owned the player has it; locked it cannot be had yet; affordable the game allows buying it now (false disables Buy and names it Too costly); price what buying costs or why it is locked, as the game words it (12,500 cr, Reach level 12). */
  vehicles?: ReadonlyArray<{ id: string; title: string; body: string; image: number; owned: boolean; locked: boolean; affordable: boolean; price: string }>;
  /** The car in view (its tile selected, its stats shown, the action for it); empty is none. */
  shown: string;
  /** The shown car's place in vehicles, from 0, read out as n of N beside the arrows (the game keeps it with shown). */
  index: number;
  /** The car the player drives now (Driving); empty is none. */
  chosen: string;
  /** The shown car's ratings, keyed rows of { id, label, value, max, text }: label as the game words it (Speed); value over max fills the bar; text the figure shown beside it (8.4). */
  stats?: ReadonlyArray<{ id: string; label: string; value: number; max: number; text: string }>;
  /** Previous car. */
  onTabPrev?: (e: UiEvent) => void;
  /** Next car. */
  onTabNext?: (e: UiEvent) => void;
  /** Back. */
  onBack?: (e: UiEvent) => void;
  /** Choose a car from its tile. */
  onChoose?: (e: UiEvent, id: string) => void;
  /** Drive the shown car. */
  onSelect?: (e: UiEvent, id: string) => void;
  /** Buy the shown car. */
  onBuy?: (e: UiEvent, id: string) => void;
}

const buttonBase = `min-h-11 flex-row flex-wrap items-center justify-center gap-1 py-[0.143rem] px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-[#1e2637eb] shadow-[inset_0px_0px_0px_1px_#ffffff3d,inset_0px_1px_0px_#ffffff99] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none hover:bg-[#2a3242eb] focus-visible:shadow-[inset_0px_0px_0px_2.288px_#ffffff,0px_0px_0px_2.288px_#060910e6] disabled:opacity-45`;
const stepButton = `size-11 items-center justify-center p-0 text-fg bg-[#1e2637eb] shadow-[inset_0px_0px_0px_1px_#ffffff3d,inset_0px_1px_0px_#ffffff99] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none hover:bg-[#2a3242eb] focus-visible:shadow-[inset_0px_0px_0px_2.288px_#ffffff,0px_0px_0px_2.288px_#060910e6]`;
const keyCap = `shrink-0 pointer-coarse:hidden bg-linear-to-b from-[#ffffff] to-[#c9d1e2] text-[#0b0f18] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] text-[0.857rem] leading-[1.5] py-0 px-1`;

/**
 * Choosing a car: the shown car large (its picture, name and class) with
 * arrows to step the cars, a strip of tiles to jump to one, the shown
 * car's ratings as bars, and one action for it: Drive when owned, Buy
 * with its price when it can be had, a disabled Locked with the reason
 * when it cannot, Driving when it is already the player's car. The game
 * owns prices, unlocking and every rating; the garage reports the car's
 * key and Back.
 */
export function RacingGarage(props: RacingGarageProps) {
  const heading = () => ((props.title ?? "") !== "" ? (props.title ?? "") : t("racing.garage.title"));
  const vehicles = () => props.vehicles ?? [];
  const stats = () => props.stats ?? [];
  const shownVehicle = () => vehicles().find((v) => v.id === props.shown);
  return (
    <View class={`relative flex-col gap-6 w-full h-full overflow-hidden bg-[#0d1119] text-fg p-6 compact:p-2`} role="group" label={heading()}>
      <Show when={shownVehicle()?.image} keyed>
        {(img) => <Image src={String(img())} class="absolute inset-0 size-full" />}
      </Show>
      <View class="absolute inset-0 pointer-events-none bg-linear-to-b from-[#060910b3] via-transparent to-[#060910eb] compact:bottom-auto compact:h-[38%]" />
      <View class="relative flex-row w-full shrink-0">
        <Text class={`flex-1 min-w-0 text-[0.857rem] leading-[1.3] font-semibold tracking-[0.14em] uppercase text-fg text-shadow-xs text-shadow-bg/55`}>{heading()}</Text>
      </View>
      <Show when={vehicles().length} fallback={
        <View class="relative flex-col items-center justify-center gap-6 w-full h-full" role="status">
          <Text class={`text-[1.143rem] text-fg-muted`}>{t("racing.garage.empty")}</Text>
          <View class={buttonBase} role="button" label={t("racing.garage.back")} onPress={(e) => props.onBack?.(e)} onAction={{ back: (e) => props.onBack?.(e) }}>
            <Text glyph="back" class={keyCap} />
            <Text>{t("racing.garage.back")}</Text>
          </View>
        </View>
      }>
        <Scroll class="relative w-full min-h-0 flex-1 flex-row flex-wrap items-end justify-between gap-3 compact:flex-col compact:flex-nowrap compact:items-stretch compact:justify-end">
          <View
            class="relative flex-col items-start min-w-0 max-w-full compact:w-full compact:shrink-0"
            onAction={{ tabPrev: (e) => props.onTabPrev?.(e), tabNext: (e) => props.onTabNext?.(e) }}
          >
            <Show when={shownVehicle()} keyed>
              {(v) => (
                <View class="flex-col gap-1 w-full" label={v().title} data-owned={v().owned} data-chosen={v().id === props.chosen} data-locked={v().locked}>
                  <Show when={!v().image}>
                    <View class={`w-full aspect-[3.2] bg-[#ffffff0a] shadow-[inset_0px_0px_0px_1px_#ffffff1a] rounded-tl-[11.43px] rounded-tr-none rounded-br-[11.43px] rounded-bl-none`} />
                  </Show>
                  <Text class={`w-full text-left text-[0.857rem] leading-[1.3] font-semibold tracking-[0.14em] uppercase text-[#38e8ff] text-shadow-xs text-shadow-bg/55`}>{v().body}</Text>
                  <Text class={`w-full text-left text-[4.714rem] leading-none font-bold italic uppercase text-fg text-shadow-xs text-shadow-bg/55 line-clamp-3 overflow-hidden`}>{v().title}</Text>
                </View>
              )}
            </Show>
            <View class="flex-row items-center gap-4">
              <View class={stepButton} role="button" label={t("racing.garage.prev")} onPress={(e) => props.onTabPrev?.(e)}>
                <Icon name="back" class="size-6 text-fg" />
              </View>
              <Text class={`min-w-16 text-center whitespace-nowrap text-[1.286rem] font-bold tabular-nums text-fg text-shadow-xs text-shadow-bg/55`}>
                {t("racing.garage.count", { n: props.index + 1, total: vehicles().length })}
              </Text>
              <View class={stepButton} role="button" label={t("racing.garage.next")} onPress={(e) => props.onTabNext?.(e)}>
                <Icon name="back" class="size-6 rotate-180 text-fg" />
              </View>
            </View>
          </View>
          <View class={`grow-[0.6] max-w-full flex-col gap-3 p-3 bg-[#1a2130] shadow-[inset_0px_0px_0px_1px_#ffffff24,inset_0px_1px_0px_#ffffff80] rounded-tl-[11.43px] rounded-tr-none rounded-br-[11.43px] rounded-bl-none compact:w-full compact:shrink-0 compact:p-2`}>
            <View class="flex-col gap-3" role="list" label={t("racing.garage.stats")}>
              <For each={stats()} key={(s) => s.id}>
                {(s) => (
                  <View class="min-w-0" role="listitem">
                    <StatBar value={s().value} max={s().max} kind="stamina" label={s().label} showValue valueText={s().text} low={0} critical={0} bare name={s().label} />
                  </View>
                )}
              </For>
            </View>
            <View class="flex-row flex-wrap justify-end gap-2 compact:w-full">
              <View class={buttonBase} role="button" label={t("racing.garage.back")} onPress={(e) => props.onBack?.(e)} onAction={{ back: (e) => props.onBack?.(e) }}>
                <Text glyph="back" class={keyCap} />
                <Text>{t("racing.garage.back")}</Text>
              </View>
              <Show when={shownVehicle()} keyed>
                {(v) => (
                  <Show
                    when={v().locked}
                    fallback={
                      <Show
                        when={v().owned}
                        fallback={
                          <View class={`min-h-11 flex-row flex-wrap items-center justify-center gap-1 py-[0.143rem] px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-[#1e2637eb] shadow-[inset_0px_0px_0px_1px_#ffffff3d,inset_0px_1px_0px_#ffffff99] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none hover:bg-[#2a3242eb] focus-visible:shadow-[inset_0px_0px_0px_2.288px_#ffffff,0px_0px_0px_2.288px_#060910e6] disabled:opacity-45 flex-1 min-w-[8.571rem]`} role="button" disabled={!v().affordable} label={t("racing.garage.buy", { price: v().price })} onPress={(e) => props.onBuy?.(e, v().id)}>
                            <Text>{t("racing.garage.buy", { price: v().price })}</Text>
                            <Show when={!v().affordable}>
                              <Text>{t("racing.garage.short")}</Text>
                            </Show>
                          </View>
                        }
                      >
                        <View class={`min-h-11 flex-row flex-wrap items-center justify-center gap-1 py-[0.143rem] px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-[#1e2637eb] shadow-[inset_0px_0px_0px_1px_#ffffff3d,inset_0px_1px_0px_#ffffff99] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none hover:bg-[#2a3242eb] focus-visible:shadow-[inset_0px_0px_0px_2.288px_#ffffff,0px_0px_0px_2.288px_#060910e6] disabled:opacity-45 flex-1 min-w-[8.571rem]`} role="button" disabled={v().id === props.chosen} label={v().id === props.chosen ? t("racing.garage.driving") : t("racing.garage.select")} onPress={(e) => props.onSelect?.(e, v().id)}>
                          <Text>{v().id === props.chosen ? t("racing.garage.driving") : t("racing.garage.select")}</Text>
                        </View>
                      </Show>
                    }
                  >
                    <View class={`min-h-11 flex-row flex-wrap items-center justify-center gap-1 py-[0.143rem] px-4 text-[0.929rem] font-bold tracking-[0.14em] uppercase text-fg bg-[#1e2637eb] shadow-[inset_0px_0px_0px_1px_#ffffff3d,inset_0px_1px_0px_#ffffff99] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none hover:bg-[#2a3242eb] focus-visible:shadow-[inset_0px_0px_0px_2.288px_#ffffff,0px_0px_0px_2.288px_#060910e6] disabled:opacity-45 flex-1 min-w-[8.571rem]`} role="button" disabled={true} description={v().price} label={t("racing.garage.locked")}>
                      <Text>{t("racing.garage.locked")}</Text>
                      <Show when={v().price}>
                        <Text>{v().price}</Text>
                      </Show>
                    </View>
                  </Show>
                )}
              </Show>
            </View>
          </View>
        </Scroll>
        <View class="relative shrink-0 min-w-0">
          <View class="flex-row items-stretch p-0.5" role="listbox" label={t("racing.garage.vehicles")}>
            <For each={vehicles()} key={(v) => v.id}>
              {(v) => (
                <View
                  class={`shrink-0 w-[12.143rem] min-h-16 px-2 py-1 bg-[#141a26] shadow-[inset_0px_0px_0px_1px_#ffffff29] rounded-tl-[6.858px] rounded-tr-none rounded-br-[6.858px] rounded-bl-none hover:shadow-[inset_0px_0px_0px_1px_#ffffff73] focus-visible:shadow-[inset_0px_0px_0px_2.288px_#ffffff,0px_0px_0px_2.288px_#060910e6] data-selected:bg-[#1a2130] data-selected:shadow-[inset_0px_0px_0px_2.288px_#38e8ff] data-locked:text-fg-muted compact:w-[8.857rem]`}
                  role="option"
                  selected={v().id === props.shown}
                  label={v().title}
                  onPress={(e) => props.onChoose?.(e, v().id)}
                  data-owned={v().owned}
                  data-chosen={v().id === props.chosen}
                  data-locked={v().locked}
                >
                  <View class="flex-col gap-1 w-full items-start">
                    <Text class={`w-full text-left text-[0.929rem] leading-[1.15] font-bold text-fg`}>{v().title}</Text>
                    <Text
                      class={`max-w-full py-0 px-1 text-[0.857rem] leading-[1.5] font-bold tracking-[0.14em] uppercase bg-[#060910e0] shadow-[inset_0px_1px_0px_#ffffff38] rounded-tl-[4.576px] rounded-tr-none rounded-br-[4.576px] rounded-bl-none text-fg data-driving:text-[#0b0f18] data-driving:bg-[#38e8ff] data-driving:shadow-none data-locked:text-fg-muted data-price:text-gold`}
                      data-driving={v().id === props.chosen}
                      data-locked={v().locked && v().id !== props.chosen}
                      data-price={!v().locked && v().id !== props.chosen && !v().owned}
                    >
                      {v().id === props.chosen ? t("racing.garage.driving") : v().locked ? t("racing.garage.locked") : v().owned ? t("racing.garage.owned") : v().price}
                    </Text>
                  </View>
                </View>
              )}
            </For>
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingGarageProps>> = {
  default: { "title": "", "shown": "gt", "index": 1, "chosen": "hatch", "vehicles": [{ "id": "hatch", "title": "Pulse Hatch", "body": "Class D · Front drive", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "gt", "title": "Corsa GT", "body": "Class B · Rear drive", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "rally", "title": "Talon Rally", "body": "Class B · All-wheel drive", "image": 0, "owned": false, "locked": false, "affordable": true, "price": "18,500 cr" }, { "id": "proto", "title": "Vanta LMP", "body": "Class S · Prototype", "image": 0, "owned": false, "locked": true, "affordable": true, "price": "Reach level 20" }], "stats": [{ "id": "speed", "label": "Speed", "value": 7.8, "max": 10, "text": "7.8" }, { "id": "accel", "label": "Acceleration", "value": 6.9, "max": 10, "text": "6.9" }, { "id": "handling", "label": "Handling", "value": 8.4, "max": 10, "text": "8.4" }, { "id": "braking", "label": "Braking", "value": 7.1, "max": 10, "text": "7.1" }] },
  selected: { "title": "", "shown": "hatch", "index": 0, "chosen": "hatch", "vehicles": [{ "id": "hatch", "title": "Pulse Hatch", "body": "Class D · Front drive", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "gt", "title": "Corsa GT", "body": "Class B · Rear drive", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }], "stats": [{ "id": "speed", "label": "Speed", "value": 5.2, "max": 10, "text": "5.2" }, { "id": "handling", "label": "Handling", "value": 7.7, "max": 10, "text": "7.7" }] },
  buy: { "title": "", "shown": "rally", "index": 1, "chosen": "hatch", "vehicles": [{ "id": "hatch", "title": "Pulse Hatch", "body": "Class D · Front drive", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "rally", "title": "Talon Rally", "body": "Class B · All-wheel drive", "image": 0, "owned": false, "locked": false, "affordable": true, "price": "18,500 cr" }], "stats": [{ "id": "speed", "label": "Speed", "value": 7.2, "max": 10, "text": "7.2" }, { "id": "handling", "label": "Handling", "value": 8.9, "max": 10, "text": "8.9" }] },
  short: { "title": "", "shown": "rally", "index": 1, "chosen": "hatch", "vehicles": [{ "id": "hatch", "title": "Pulse Hatch", "body": "Class D · Front drive", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "rally", "title": "Talon Rally", "body": "Class B · All-wheel drive", "image": 0, "owned": false, "locked": false, "affordable": false, "price": "18,500 cr" }], "stats": [{ "id": "speed", "label": "Speed", "value": 7.2, "max": 10, "text": "7.2" }, { "id": "handling", "label": "Handling", "value": 8.9, "max": 10, "text": "8.9" }] },
  locked: { "title": "", "shown": "proto", "index": 1, "chosen": "hatch", "vehicles": [{ "id": "hatch", "title": "Pulse Hatch", "body": "Class D · Front drive", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "proto", "title": "Vanta LMP", "body": "Class S · Prototype", "image": 0, "owned": false, "locked": true, "affordable": true, "price": "Reach level 20" }], "stats": [{ "id": "speed", "label": "Speed", "value": 9.8, "max": 10, "text": "9.8" }, { "id": "handling", "label": "Handling", "value": 9.1, "max": 10, "text": "9.1" }] },
  empty: { "title": "", "shown": "", "index": 0, "chosen": "", "vehicles": [], "stats": [] },
  overflow: { "title": "Garage of the Grand Touring Championship", "shown": "v7", "index": 6, "chosen": "v1", "vehicles": [{ "id": "v1", "title": "Pulse Hatch", "body": "Class D", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "v2", "title": "Corsa GT", "body": "Class B", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "v3", "title": "Talon Rally", "body": "Class B", "image": 0, "owned": false, "locked": false, "affordable": true, "price": "18,500 cr" }, { "id": "v4", "title": "Vanta LMP", "body": "Class S", "image": 0, "owned": false, "locked": true, "affordable": true, "price": "Reach level 20" }, { "id": "v5", "title": "Kestrel Roadster", "body": "Class C", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "v6", "title": "Basalt Truck", "body": "Class D", "image": 0, "owned": false, "locked": false, "affordable": true, "price": "9,900 cr" }, { "id": "v7", "title": "Meridian Hyperion Twin-Turbo Competizione Evoluzione", "body": "Class S · Mid-engine · Carbon monocoque · Limited series", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }, { "id": "v8", "title": "Nimbus EV", "body": "Class A", "image": 0, "owned": false, "locked": true, "affordable": true, "price": "Win the Coastal Cup" }, { "id": "v9", "title": "Ridge Buggy", "body": "Class C", "image": 0, "owned": false, "locked": false, "affordable": true, "price": "6,200 cr" }, { "id": "v10", "title": "Solace Coupe", "body": "Class A", "image": 0, "owned": true, "locked": false, "affordable": true, "price": "" }], "stats": [{ "id": "speed", "label": "Top speed", "value": 9.9, "max": 10, "text": "9.9" }, { "id": "accel", "label": "Acceleration", "value": 9.6, "max": 10, "text": "9.6" }, { "id": "handling", "label": "Handling", "value": 7.4, "max": 10, "text": "7.4" }, { "id": "braking", "label": "Braking", "value": 8.8, "max": 10, "text": "8.8" }, { "id": "offroad", "label": "Off-road", "value": 1.2, "max": 10, "text": "1.2" }] },
};
