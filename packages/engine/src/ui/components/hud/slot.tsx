import type { Texture } from "@origingame/engine";
import { For, Icon, Image, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./slot.en.ogt";

export interface HudSlotProps {
  /** What a reader calls it (Ember slash). */
  name: string;
  /** Its art, a texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: Texture | 0;
  /** An icon of the atlas, drawn when there is no art; empty is none. */
  glyph?: string;
  /** The input as the game names it (1, Q), on a key cap under the slot; empty is none. */
  key?: string;
  /** skill, item (a consumable, its count on a tag) or ultimate (a hexagon inside a ring, gold by default, that fills with its charge). */
  kind?: string;
  /** The share of its cooldown still to run, 0 to 1: a dark sweep with a lit leading edge; 0 is none. */
  cooldown?: number;
  /** The cooldown left as the game words it (12), over the sweep; empty is none. */
  time?: string;
  /** How many are left, on a tag at the slot's corner; 0 shows none. */
  count?: number;
  /** Ready and worth pressing now: a lit edge and a glow (an ultimate at full charge has them without it). */
  ready?: boolean;
  /** It cannot be paid for: the art greyed under a veil. */
  unaffordable?: boolean;
  /** Why it cannot be used (No mana), on the slot while it is unaffordable and not cooling; empty is none. */
  reason?: string;
  /** Charges held, the lit pips over the slot. */
  charges?: number;
  /** Charges it holds; 0 draws no pips. */
  chargesMax?: number;
  /** An ultimate's charge, 0 to 1: its ring's lit arc and, under 1, the percent on it. */
  charge?: number;
  /** Extra classes for the root, to size it (NSH style). */
  class?: string;
}

/**
 * One slot of a HUD's skill bar, a block a bar is built from: the art in
 * an octagon with a lit edge, its cooldown a dark sweep with a lit leading
 * edge and the time left, a slot that cannot be paid for greyed with its
 * reason on it, charge pips over a slot that holds charges, a consumable's
 * count on a tag, and the ultimate in a hexagon inside a gold ring that
 * fills with its charge; a key cap under it. Its side is --hud-slot-size.
 * The game owns every cooldown, cost and charge; read only. The style cuts
 * the cell with clip-path; here the cell is rounded and the sweep and the
 * ring are pack art (a nine-disc wipe and a nine-ring wipe).
 */
export function HudSlot(props: HudSlotProps) {
  const cooldown = () => Math.min(Math.max(props.cooldown ?? 0, 0), 1);
  const charge = () => Math.min(Math.max(props.charge ?? 0, 0), 1);
  const ultimate = () => (props.kind ?? "skill") === "ultimate";
  const charges = () => props.charges ?? 0;
  const chargesMax = () => props.chargesMax ?? 0;
  const ready = () => (props.ready ?? false) || (ultimate() && charge() >= 1);
  const cooling = () => cooldown() > 0 || (ultimate() && charge() < 1);
  const unaffordable = () => props.unaffordable ?? false;
  return (
    <View
      class={`relative w-fit max-w-full flex-col items-center gap-y-1.25 ${props.class ?? ""}`}
      role="group"
      label={unaffordable() && (props.reason ?? "") !== "" ? t("hud.slot.state", { name: props.name, state: props.reason }) : props.name}
      data-ultimate={ultimate()}
      data-ready={ready()}
      data-unaffordable={unaffordable()}
      data-cooling={cooling()}
    >
      <Show when={chargesMax() > 0}>
        <View
          class="flex-row gap-x-0.75"
          role="meter"
          label={t("hud.slot.charges")}
          valueMin={0}
          valueMax={chargesMax()}
          valueNow={charges() <= 0 ? 0 : charges()}
        >
          <For each={Array.from({ length: chargesMax() }, (_, i) => i)} key={(i) => i}>
            {(i) => (
              <View class="relative w-3 h-1.25 rounded-control bg-sheen overflow-hidden">
                <View
                  class="absolute inset-y-0 left-0 bg-hud-accent shadow-glow-sm"
                  style={{ width: `${Math.min(Math.max(charges() - i(), 0), 1) * 100}%` }}
                />
              </View>
            )}
          </For>
        </View>
      </Show>
      <View
        class={`relative size-12.75 ${ready() ? "shadow-glow" : ""} ${ready() && ultimate() ? "shadow-glow shadow-gold" : ""} ${packClass("Slot", "frame")}`}
      >
        <Show when={ultimate()}>
          <View
            class="absolute inset-0 rounded-full border-3 border-rim overflow-hidden"
            role="meter"
            label={t("hud.slot.charge")}
            valueMin={0}
            valueMax={100}
            valueNow={charge() <= 0 ? 0 : charge() >= 1 ? 100 : charge() * 100}
          >
            <View class="absolute inset-0 nine-ring bg-gold" wipe={Math.min(Math.max(charge(), 0), 1)} />
          </View>
        </Show>
        <View
          class={`absolute inset-0.5 rounded-field data-ultimate:inset-1.5 data-ultimate:rounded-capsule overflow-hidden ${ready() ? "bg-linear-to-b from-hud-accent-hi to-hud-accent" : "bg-tile"}`}
          data-ultimate={ultimate()}
        >
          <Show when={(props.icon ?? 0) > 0} fallback={
            <View class="absolute inset-0 flex-col items-center justify-center bg-linear-to-b from-raised-lo to-surface">
              <Show when={(props.glyph ?? "") !== ""}>
                <Icon class={`size-1/2 text-fg text-shadow-tag ${cooling() || unaffordable() ? "opacity-45" : ""}`} name={props.glyph} />
              </Show>
            </View>
          }>
            <Image class={`absolute inset-0 size-full ${cooling() || unaffordable() ? "opacity-45" : ""}`} src={props.icon ?? 0} />
          </Show>
          <Show when={unaffordable()}>
            <View class="absolute inset-0 bg-surface" />
          </Show>
          <Show when={cooling() && !unaffordable()}>
            <View class="absolute inset-0 nine-disc bg-scrim" wipe={Math.min(cooldown(), 1)} wipeCcw />
          </Show>
          <Show when={cooling() && (props.time ?? "") !== ""}>
            <View class="absolute inset-0 flex-row items-center justify-center">
              <Text class={`${ultimate() ? "text-base" : "text-title"} leading-none font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg text-shadow-tag`}>{props.time}</Text>
            </View>
          </Show>
          <Show when={unaffordable() && (props.reason ?? "") !== "" && cooldown() <= 0}>
            <View class="absolute inset-0 flex-row items-center justify-center px-0.5">
              <Text class="max-w-full max-h-21/25 text-center text-xs leading-none font-bold tracking-normal uppercase whitespace-nowrap overflow-hidden text-mana text-shadow-tag">{props.reason}</Text>
            </View>
          </Show>
          <Show when={ultimate() && charge() < 1}>
            <View class="absolute inset-0 flex-row items-center justify-center">
              <Text class="text-base leading-none font-bold tabular-nums whitespace-nowrap text-fg text-stroke-3 text-stroke-bg text-shadow-tag">{`${formatNumber(charge() * 100, { decimals: 0 })}%`}</Text>
            </View>
          </Show>
          <Show when={(props.count ?? 0) > 0}>
            <View class="absolute top-0 right-0">
              <Text class="max-w-15.25 -mt-1.5 -mr-1 bg-surface inset-shadow-lit px-1.25 text-xs font-bold tabular-nums whitespace-nowrap leading-normal text-fg rounded-bl-control overflow-hidden">
                {t("hud.slot.count", { count: formatNumber(props.count ?? 0) })}
              </Text>
            </View>
          </Show>
        </View>
      </View>
      <Show when={(props.key ?? "") !== ""}>
        <Text
          class={`min-w-6 max-w-16.75 px-1 pointer-coarse:hidden text-xs leading-loose font-bold text-center rounded-control shadow-key text-shadow-none whitespace-nowrap overflow-hidden ${ultimate() ? "text-gold-fg bg-linear-to-b from-gold-hi to-gold" : "text-key-fg bg-linear-to-b from-key-hi to-key-lo"}`}
        >
          {props.key}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudSlotProps>> = {
  default: {"name":"Ember slash","icon":0,"glyph":"sword","key":"1","kind":"skill","cooldown":0,"time":"","count":0,"ready":true,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},
  cooling: {"name":"Arc step","icon":0,"glyph":"star","key":"3","kind":"skill","cooldown":0.4,"time":"3","count":0,"ready":false,"unaffordable":false,"reason":"","charges":1,"chargesMax":3,"charge":0},
  item: {"name":"Healing tonic","icon":0,"glyph":"potion","key":"Z","kind":"item","cooldown":0,"time":"","count":5,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},
  ultimate: {"name":"Cataclysm","icon":0,"glyph":"fire","key":"R","kind":"ultimate","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0.58},
  unaffordable: {"name":"Void rend","icon":0,"glyph":"skull","key":"4","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":true,"reason":"No mana","charges":0,"chargesMax":0,"charge":0},
  empty: {"name":"","icon":0,"glyph":"","key":"","kind":"skill","cooldown":0,"time":"","count":0,"ready":false,"unaffordable":false,"reason":"","charges":0,"chargesMax":0,"charge":0},
  overflow: {"name":"Thunderous riposte of the Ninefold Gate","icon":0,"glyph":"bolt","key":"Right Trigger","kind":"skill","cooldown":0.5,"time":"1:02:44","count":12345,"ready":false,"unaffordable":false,"reason":"","charges":7,"chargesMax":9,"charge":0},
};
