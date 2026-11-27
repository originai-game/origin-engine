import { For, Icon, Show, Text, View, formatDuration, formatNumber, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./ability-slot.en.strings.ts";

export interface AbilitySlotProps {
  /** An icon of the atlas, drawn large on a gradient. */
  icon: string;
  /** What a screen reader calls it (Fireball). */
  name: string;
  /** The action whose glyph the hotkey cap shows on the device used last (it follows a rebinding); empty is none. */
  action: string;
  /** A label for the cap instead of the action's glyph (Q, LB); empty shows the action's. */
  hotkey: string;
  /** Seconds of cooldown left; 0 is ready. */
  cooldown: number;
  /** The cooldown's length; the sweep shows cooldown over it. Default 1. */
  cooldownMax?: number;
  /** Charges ready, in a corner badge when maxCharges is over 1. */
  charges: number;
  /** Charges it holds; 0 or 1 is none. */
  maxCharges: number;
  /** How many are left (a quick item: potions, scrolls), at the bottom's end; 0 shows none. */
  count: number;
  /** Its cost (40) at the bottom left while ready; empty is none. */
  cost: string;
  /** Not enough resource: the art tinted red and the cost in danger. */
  unaffordable?: boolean;
  /** The target is out of range: the art tinted red. */
  outOfRange?: boolean;
  /** Toggled on or channelling: a breathing glow and a primary edge. */
  active?: boolean;
  /** Not learnt yet: dark art under a lock. */
  locked?: boolean;
  /** Cannot be used now: the art washed out under a red slash. */
  disabled?: boolean;
  /** An ultimate's charge 0 to 1: larger, a gold ring filling round it and a percent; negative is a plain slot. Default -1. */
  ultimate?: number;
  /** Ranks learnt (pips right under it). */
  level: number;
  /** It fills the place it is given (a strip's), rather than standing at the theme's ability size. */
  fit?: boolean;
  /** Ranks it has; 0 draws no pips. */
  maxLevel: number;
  /** Use. */
  onUse?: (e: UiEvent) => void;
}

/**
 * One ability or quick item: its icon on its face, the hotkey's cap, a
 * cooldown sweep, charges, a count, its cost, ranks as pips and its states
 * (unaffordable, out of range, active, locked, disabled, an ultimate's
 * charge ring with its percent, then Ready). Every part is a pack slot of
 * the Ability component, so a theme sets its size, shape, art, ring, glow,
 * sweep and type. It reports onUse; the game decides whether it fires.
 */
export function AbilitySlot(props: AbilitySlotProps) {
  const ult = () => (props.ultimate ?? -1) >= 0;
  const charge = () => Math.min(Math.max(props.ultimate ?? 0, 0), 1);
  const full = () => ult() && charge() >= 1;
  const cooling = () => props.cooldown > 0;
  const off = () => props.locked || props.disabled;
  return (
    <View class="max-w-full items-center gap-0.5 data-fit:w-full data-ultimate:gap-y-3" data-fit={props.fit} data-ultimate={ult()}>
      <View
        class={`relative ${packClass("Ability", "box")} ${ult() ? packClass("Ability", "box", "ultimate") : ""} ${ult() ? packClass("Ability", "frame", "ultimate") : packClass("Ability", "frame")} ${props.active && !ult() ? packClass("Ability", "frame", "ready") : ""} ${off() ? packClass("Ability", "frame", "disabled") : ""} data-fit:w-full data-fit:h-auto data-fit:aspect-square`}
        role="button"
        onPress={(e) => props.onUse?.(e)}
        label={props.name}
        valueText={props.locked ? t("component.ability-slot.locked") : props.disabled ? t("component.ability-slot.unavailable") : cooling() ? formatNumber(props.cooldown, { decimals: 1 }) : t("component.ability-slot.ready")}
        disabled={off()}
        data-fit={props.fit}
      >
        <Show when={ult()}>
          <View class={`absolute inset-0 ${packClass("Ability", "ring")} ${full() || props.active ? packClass("Ability", "ring", "ready") : ""}`} role="meter" valueMin={0} valueMax={100} valueNow={charge() * 100}>
            <View class={`absolute inset-0 ${packClass("Ability", "ring", "fill")}`} wipe={charge()} />
          </View>
        </Show>
        <View class={`absolute items-center justify-center ${packClass("Ability", "face")} ${ult() ? packClass("Ability", "face", "ultimate") : ""}`}>
          <Icon class={`${packClass("Ability", "icon")} ${off() || props.unaffordable || props.outOfRange ? packClass("Ability", "icon", "disabled") : ""}`} name={props.locked ? "lock" : props.icon} />
          <Show when={props.unaffordable || props.outOfRange}>
            <View class={`absolute inset-0 ${packClass("Ability", "tint")}`} />
          </Show>
          <Show when={cooling()}>
            <View class={`absolute inset-0 ${packClass("Ability", "sweep")} ${ult() ? packClass("Ability", "sweep", "ultimate") : ""}`} wipe={Math.min(Math.max((props.cooldownMax ?? 1) > 0 ? props.cooldown / (props.cooldownMax ?? 1) : 1, 0), 1)} wipeCcw />
          </Show>
          <Show when={props.disabled && !props.locked}>
            <View class="absolute inset-0 items-center justify-center">
              <View class={`shrink-0 ${packClass("Ability", "slash")}`} />
            </View>
          </Show>
        </View>
        <Show when={cooling()}>
          <View class="absolute inset-0 items-center justify-center pointer-events-none">
            <Text class={`whitespace-nowrap ${packClass("Ability", "count")}`}>
              {props.cooldown < 10 ? formatNumber(props.cooldown, { decimals: 1 }) : formatDuration(props.cooldown)}
            </Text>
          </View>
        </Show>
        <Show when={ult() && !cooling() && !off()}>
          <View class="absolute inset-x-0 -bottom-2 flex-row justify-center pointer-events-none">
            <Show when={!full()} fallback={<Text class={`whitespace-nowrap ${packClass("Ability", "chip", "ready")}`}>{t("component.ability-slot.ready")}</Text>}>
              <Text class={`whitespace-nowrap ${packClass("Ability", "chip", "ult")}`}>{`${formatNumber(charge() * 100, { decimals: 0 })}%`}</Text>
            </Show>
          </View>
        </Show>
        <Show when={props.cost && !cooling() && !props.locked && !ult()}>
          <Text class={`absolute bottom-0.5 left-0.5 whitespace-nowrap ${packClass("Ability", "chip", "cost")} ${props.unaffordable ? packClass("Ability", "chip", "short") : ""}`}>{props.cost}</Text>
        </Show>
        <Show when={props.count > 0}>
          <Text class={`absolute right-1.5 bottom-1.5 ${packClass("Ability", "chip", "count")}`}>{formatNumber(props.count)}</Text>
        </Show>
        <Show when={props.maxCharges > 1 && !props.locked}>
          <Text class={`absolute -top-1.5 -right-1.5 whitespace-nowrap ${packClass("Ability", "chip", "charges")}`}>{formatNumber(props.charges)}</Text>
        </Show>
      </View>
      <Show when={props.maxLevel > 0}>
        <View class="flex-row gap-0.5" role="meter" valueMin={0} valueMax={props.maxLevel} valueNow={props.level}>
          <For each={Array.from({ length: props.maxLevel }, (_, i) => i)}>
            {(i) => <View class={`${packClass("Ability", "pip")} ${i() < props.level ? packClass("Ability", "pip", "on") : ""}`} />}
          </For>
        </View>
      </Show>
      <Show when={props.hotkey || props.action}>
        <Text class={`max-w-full whitespace-nowrap overflow-hidden ${packClass("Ability", "chip", "key")}`} glyph={props.hotkey ? undefined : props.action}>
          {props.hotkey}
        </Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<AbilitySlotProps>> = {
  default: {"icon":"fire","name":"Fireball","action":"","hotkey":"Q","cooldown":4.2,"cooldownMax":8,"charges":2,"maxCharges":3,"count":0,"cost":"40","unaffordable":false,"outOfRange":false,"active":false,"locked":false,"disabled":false,"ultimate":-1,"level":2,"maxLevel":3},
  ultimate: {"icon":"star","name":"Overdrive","action":"","hotkey":"R","cooldown":0,"charges":0,"maxCharges":0,"count":0,"cost":"","ultimate":0.62,"level":0,"maxLevel":0},
  ready: {"icon":"star","name":"Overdrive","action":"","hotkey":"R","cooldown":0,"charges":0,"maxCharges":0,"count":0,"cost":"","ultimate":1,"level":0,"maxLevel":0},
};
