import { For, Icon, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import { Spinner } from "../base/spinner";
import "./pickup-feed.en.ogt";

export interface PickupFeedItem {
  id: string;
  /** Its name (Iron ore). */
  name: string;
  /** An icon of the atlas; empty is none. */
  icon?: string;
  /** A signed change. */
  quantity?: number;
  /** Its caption (Crafting material); empty is none. */
  detail?: string;
}

export interface PickupFeedProps {
  /** The feed heading; empty uses Pickups. */
  label?: string;
  /** Keyed entries. The game owns ordering, expiry and retention; quantity is a signed change. No timers run in this document. */
  items?: ReadonlyArray<PickupFeedItem>;
  /** Waiting for pickup data; stale entries are hidden. */
  loading?: boolean;
}

/**
 * A HUD plate of what the player just picked up: its heading, then its
 * entries, each its icon, name and caption with the signed change in its
 * trend's colour; the game owns ordering, expiry and retention, and the
 * empty state says so. Read only.
 */
export function PickupFeed(props: PickupFeedProps) {
  return (
    <View class={`flex-col max-w-full gap-y-[0.286rem] border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border p-2`} label={(props.label ?? "") !== "" ? props.label : t("hud.pickup-feed.name")} busy={props.loading}>
      <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg-muted text-shadow-2xs">{(props.label ?? "") !== "" ? props.label : t("hud.pickup-feed.name")}</Text>
      <Show when={props.loading} fallback={
        <>
          <View class="flex-col gap-y-[0.143rem]" role="log" live="polite">
            <For each={props.items ?? []} key={(item) => item.id}>
              {(item) => (
                <View class="flex-row items-start gap-x-2">
                  <Show when={item().icon}>
                    <Icon class="shrink-0 size-4 text-fg" name={item().icon} />
                  </Show>
                  <View class="flex-col flex-1 min-w-0 gap-y-[0.143rem]">
                    <Text class="text-[0.929rem] whitespace-nowrap truncate text-fg text-shadow-2xs">{item().name}</Text>
                    <Show when={item().detail}>
                      <Text class="text-[0.857rem] text-fg-muted text-shadow-2xs truncate">{item().detail}</Text>
                    </Show>
                  </View>
                  <Text class="shrink-0 text-[0.929rem] font-bold tabular-nums whitespace-nowrap text-[#45e08f] text-shadow-2xs data-down:text-[#ff9599]" data-down={(item().quantity ?? 0) < 0}>
                    {(item().quantity ?? 0) > 0 ? "+" : ""}{formatNumber(item().quantity ?? 0)}
                  </Text>
                </View>
              )}
            </For>
          </View>
          <Show when={(props.items ?? []).length === 0}>
            <Text class="text-[0.857rem] text-fg-muted text-shadow-2xs">{t("hud.pickup-feed.empty")}</Text>
          </Show>
        </>
      }>
        <View class="flex-row items-center gap-x-2">
          <Spinner name={t("hud.pickup-feed.loading")} />
          <Text class="text-[0.857rem] text-fg-muted text-shadow-2xs" role="status">{t("hud.pickup-feed.loading")}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<PickupFeedProps>> = {
  default: {"label":"Recovered supplies","items":[{"id":"ore-7","name":"Iron ore","icon":"gem","quantity":12,"detail":"Crafting material"},{"id":"key-2","name":"Observatory key","icon":"key","quantity":1,"detail":"Quest item"},{"id":"spent","name":"Gold","icon":"coin","quantity":-40,"detail":"Supplies purchased"}],"loading":false},
  loading: {"loading":true,"items":[]},
  overflow: {"items":[{"id":"relic","name":"Fragment of the northern observatory mechanism","icon":"gem","quantity":123456,"detail":"Recovered from the abandoned expedition storehouse"}],"loading":false},
};
