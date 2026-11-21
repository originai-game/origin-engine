import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./idle.en.ogt";

export interface IdleOfflineProps {
  /** The panel's heading; empty takes the group's (Welcome back). */
  title: string;
  /** How long the player was away, as the game words it (it wins; empty shows seconds as a duration: 1h 2m). */
  away: string;
  /** How long the player was away, in seconds, when away is empty. */
  seconds: number;
  /** What was gathered while away, keyed rows of { id, icon, name, amount }: amount as the game formats it (+12.4K). */
  earned?: ReadonlyArray<{ id: string; icon: string; name: string; amount: string }>;
  /** The game offers to double the haul (an ad, a gem); false hides Double. */
  canDouble?: boolean;
  /** What doubling costs or asks, as the game words it (Watch an ad); empty is none. */
  doubleText: string;
  /** The collect event, with the row's key where it comes from a row. */
  onCollect?: (e: UiEvent, key?: string) => void;
  /** The double event, with the row's key where it comes from a row. */
  onDouble?: (e: UiEvent, key?: string) => void;
}

/**
 * The welcome-back panel of an idle game: how long the player was away,
 * large (the game's words, or its seconds as a duration), then what was
 * gathered as sockets glowing gold from their foot, each with its art, the
 * amount and its name. Collect reports collect; Double, the game's gold
 * offer and shown only when it sets canDouble, reports double with what it
 * asks under its word. The game computes the haul and applies it. Many
 * currencies wrap and the haul scrolls.
 */
export function IdleOffline(props: IdleOfflineProps) {
  return (
    <View class={`border-rim nine-panel-cut bg-glass-hi text-fg w-full max-w-105 max-h-full p-4 gap-3 border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title !== '' ? props.title : t("idle.offline.name")}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class={`min-h-12 border-b border-rim-soft w-full items-start py-1 px-4 gap-0`}>
        <Text class={`whitespace-nowrap text-ellipsis text-xs font-semibold tracking-kicker uppercase text-fg-muted overflow-hidden`}>
          {t("idle.offline.kicker")}
        </Text>
        <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg text-start m-0`}>
          {props.title !== '' ? props.title : t("idle.offline.name")}
        </Text>
      </View>
      <View class={`flex-row flex-wrap items-baseline gap-1`} role="status" label={t("idle.offline.away", { time: props.away !== '' ? props.away : formatDuration(props.seconds) })}>
        <Text class={`tabular-nums text-fg text-[2.286rem] font-bold shrink`}>
          {props.away !== '' ? props.away : formatDuration(props.seconds)}
        </Text>
        <Text class={`tracking-xs text-fg-muted text-xs font-semibold shrink`}>
          {t("idle.offline.awayLabel")}
        </Text>
      </View>
      <Show when={(props.earned ?? []).length} fallback={
          <Text class={`tracking-xs text-fg-muted text-center p-4 m-0`} role="status">
            {t("idle.offline.empty")}
          </Text>
      }>
        <Scroll class="shrink gap-0 ">
          <View class="flex-row flex-wrap items-center w-full p-0 m-0 gap-1" label={t("idle.offline.earned")}>
            <For each={props.earned}>
              {(e) => (
              <View class={`w-full grow shrink basis-0 items-center min-w-27.25 max-w-48 bg-tile border border-rim-soft nine-plate-cut p-1 m-0 gap-0 compact:min-w-22.5`} label={t("idle.offline.row", { name: e().name, amount: e().amount })}>
                <View class={`grow shrink basis-0 justify-center items-center gap-2 size-12`}>
                  <Icon class="size-full" name={e().icon} />
                </View>
                <Text class={`max-w-full whitespace-nowrap text-ellipsis text-lg font-bold tabular-nums text-fg overflow-hidden`}>
                  {e().amount}
                </Text>
                <Text class={`text-fg-muted max-w-full whitespace-nowrap text-ellipsis text-xs font-semibold overflow-hidden`}>
                  {e().name}
                </Text>
              </View>
              )}
            </For>
          </View>
        </Scroll>
      </Show>
      <View class="flex-row flex-wrap items-center w-full justify-end gap-2">
        <Show when={props.canDouble}>
          <Button class={`text-gold-fg bg-linear-to-b from-gold-hi to-gold min-h-11 min-w-30 max-w-full gap-1 active:bg-linear-to-b active:from-gold-hi active:to-gold active:bg-transparent hover:bg-linear-to-b hover:from-gold-hi hover:to-gold hover:bg-transparent compact:min-w-0 compact:grow compact:shrink compact:basis-0 pointer-coarse:grow pointer-coarse:shrink pointer-coarse:basis-0 ${packClass("Button", "plate")}`} onPress={(e) => props.onDouble?.(e)}>
            <View class="items-center max-w-full gap-0">
              <Text>{t("idle.offline.double")}</Text>
              <Show when={props.doubleText !== ''}>
                <Text class="whitespace-nowrap text-ellipsis max-w-full text-xs overflow-hidden">
                  {props.doubleText}
                </Text>
              </Show>
            </View>
          </Button>
        </Show>
        <Button class={`text-accent min-h-11 min-w-30 max-w-full gap-1 compact:min-w-0 compact:grow compact:shrink compact:basis-0 pointer-coarse:grow pointer-coarse:shrink pointer-coarse:basis-0 ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onCollect?.(e)}>
          <Text>{t("idle.offline.collect")}</Text>
        </Button>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<IdleOfflineProps>> = {
  default: {"title":"","away":"","seconds":13530,"earned":[{"id":"gold","icon":"coin","name":"Gold","amount":"+1.24M"},{"id":"wood","icon":"tree","name":"Wood","amount":"+5,000"},{"id":"gems","icon":"gem","name":"Gems","amount":"+3"}],"canDouble":true,"doubleText":"Watch an ad"},
  disabled: {"title":"","away":"8h (capped)","seconds":0,"earned":[{"id":"gold","icon":"coin","name":"Gold","amount":"+880K"}],"canDouble":false,"doubleText":""},
  empty: {"title":"","away":"","seconds":42,"earned":[],"canDouble":false,"doubleText":""},
  overflow: {"title":"While You Slumbered, the Clockwork Empire Kept Turning","away":"3 days, 14 hours, 52 minutes and 9 seconds (capped at 72 hours)","seconds":0,"earned":[{"id":"gold","icon":"coin","name":"Gold Sovereigns of the Eastern Treasury","amount":"+123.45 quattuordecillion"},{"id":"wood","icon":"tree","name":"Wood","amount":"+5,000"},{"id":"gems","icon":"gem","name":"Gems","amount":"+99,999"},{"id":"stone","icon":"shield","name":"Stone","amount":"+48K"},{"id":"mana","icon":"drop","name":"Mana","amount":"+250"},{"id":"souls","icon":"skull","name":"Souls","amount":"+7"},{"id":"stars","icon":"star","name":"Stardust","amount":"+1.5B"}],"canDouble":true,"doubleText":"Spend 50 gems to double everything gathered"},
};
