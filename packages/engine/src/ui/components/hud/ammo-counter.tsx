import { For, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import "./ammo-counter.en.ogt";

export interface AmmoCounterProps {
  /** The weapon's name. */
  weapon: string;
  /** Its fire mode: auto, burst or single (beside the name in words, and as three leaning pips: all lit for auto, two for burst, one for single). Default auto. */
  mode?: string;
  /** Rounds in the magazine. */
  magazine: number;
  /** Rounds a full magazine holds: a leaning tick per round under the count, lit for each round left (a continuous strip past 60). */
  capacity: number;
  /** Rounds carried beyond the magazine. */
  reserve: number;
  /** The reload's progress, 0 to 1; negative while not reloading. Default -1. */
  reloading?: number;
  /** The fraction of capacity at or under which the count turns low (0.25). Default 0.25. */
  low?: number;
  /** The weapon held in reserve, on a quiet line over the readout; empty hides the line. */
  secondary?: string;
  /** The reserve weapon's rounds; negative hides the figure. Default -1. */
  secondaryAmmo?: number;
  /** The input that swaps to it as the game names it (2), on a key cap; empty hides it. */
  secondaryKey?: string;
}

/**
 * Ammunition at the screen's bottom corner: the reserve weapon on a quiet
 * line, the magazine as the HUD's largest figure with the reserve beside
 * it, a tick per round under the count (lit while loaded), and the fire
 * mode as pips beside the weapon's name. Low and empty change the count's
 * ink and say so; a reload shows its progress. The game owns every count;
 * read only. The ticks' mask and the lean (skewX) have no TSX surface:
 * the ticks are separate views, the lean is dropped.
 */
export function AmmoCounter(props: AmmoCounterProps) {
  const low = () => props.magazine > 0 && props.magazine <= props.capacity * (props.low ?? 0.25);
  const empty = () => props.magazine <= 0;
  const lit = (i: number) =>
    (props.mode ?? "auto") == "auto" || ((props.mode ?? "auto") == "burst" && i < 2) || ((props.mode ?? "auto") == "single" && i < 1);
  return (
    <View
      class={`flex-col items-end gap-y-[0.357rem] w-[16.429rem] max-w-full text-fg compact:w-auto compact:gap-y-[0.143rem]`}
      role="group"
      label={t("shooter.ammo.name")}
      valueText={t("shooter.ammo.value", { magazine: props.magazine, capacity: props.capacity, reserve: props.reserve })}
      data-low={low()}
      data-empty={empty()}
    >
      <Show when={(props.secondary ?? "") != ""}>
        <View class="flex-row items-center gap-1 max-w-full min-w-0 opacity-80">
          <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/78 whitespace-nowrap truncate text-shadow-2xs text-stroke-3 text-stroke-bg min-w-0">{props.secondary}</Text>
          <Show when={(props.secondaryAmmo ?? -1) >= 0}>
            <Text class="shrink-0 text-[1.143rem] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-[0.929rem]">{formatNumber(props.secondaryAmmo ?? 0)}</Text>
          </Show>
          <Show when={(props.secondaryKey ?? "") != ""}>
            <Text class={`shrink max-w-1/2 min-w-[1.429rem] py-0 px-1 text-[0.857rem] leading-[1.5] font-bold text-center text-[#262b38] bg-linear-to-b from-white to-[#d8dbe2] rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] whitespace-nowrap truncate pointer-coarse:hidden`}>
              {props.secondaryKey}
            </Text>
          </Show>
        </View>
      </Show>
      <View class="flex-row items-baseline gap-2 max-w-full">
        <Text class="min-w-0 text-[5.429rem] leading-[0.9] font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg data-low:text-[#ffb547] data-empty:text-[#ff4d5e] compact:text-[2.571rem]" data-low={low()} data-empty={empty()}>
          {formatNumber(props.magazine)}
        </Text>
        <Text class="shrink-0 text-[2rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg/66 text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-[1.143rem]">{formatNumber(props.reserve)}</Text>
      </View>
      <Show when={props.capacity > 0}>
        <Show
          when={props.capacity > 60}
          fallback={
            <View class="flex-row gap-x-[0.143rem] w-full max-w-full h-[1rem] compact:h-[0.429rem]">
              <For each={Array.from({ length: Math.min(props.capacity, 60) }, (_, i) => i)}>
                {(i) => <View class="flex-1 h-full bg-white/22 data-on:bg-white" data-on={i() < props.magazine} />}
              </For>
            </View>
          }
        >
          <View class="relative w-full max-w-full h-[0.429rem] bg-white/22">
            <View class="absolute left-0 top-0 h-full bg-white transition-layout duration-180 ease-out" style={{ width: `${Math.min(Math.max(props.magazine / props.capacity, 0), 1) * 100}%` }} />
          </View>
        </Show>
      </Show>
      <View class="flex-row items-center gap-1 max-w-full min-w-0 compact:flex-wrap compact:justify-end">
        <View class="flex-row shrink-0 gap-x-[0.214rem] compact:hidden" data-auto={(props.mode ?? "auto") == "auto"} data-burst={(props.mode ?? "auto") == "burst"} data-single={(props.mode ?? "auto") == "single"}>
          <For each={[0, 1, 2]}>
            {(i) => <View class="w-[0.714rem] h-[0.286rem] bg-white/22 data-lit:bg-hud data-lit:shadow-[0px_0px_0.429rem_#38e8ff]" data-lit={lit(i())} />}
          </For>
        </View>
        <Text class="min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg whitespace-nowrap truncate text-shadow-2xs text-stroke-3 text-stroke-bg">{props.weapon}</Text>
        <Show when={(props.mode ?? "auto") == "auto"}>
          <Text class="shrink-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/66 text-shadow-2xs text-stroke-3 text-stroke-bg">{t("shooter.ammo.auto")}</Text>
        </Show>
        <Show when={(props.mode ?? "auto") == "burst"}>
          <Text class="shrink-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/66 text-shadow-2xs text-stroke-3 text-stroke-bg">{t("shooter.ammo.burst")}</Text>
        </Show>
        <Show when={(props.mode ?? "auto") == "single"}>
          <Text class="shrink-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/66 text-shadow-2xs text-stroke-3 text-stroke-bg">{t("shooter.ammo.single")}</Text>
        </Show>
      </View>
      <Show when={(props.reloading ?? -1) >= 0}>
        <View class="flex-row items-center gap-1 w-full">
          <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg/78 whitespace-nowrap text-shadow-2xs text-stroke-3 text-stroke-bg">{t("shooter.ammo.reloading")}</Text>
          <View class="flex-1 h-[0.286rem] overflow-hidden bg-tile shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_0px_#ffffff24]">
            <View class="h-full bg-hud transition-layout duration-180 ease-out" style={{ width: `${Math.min(props.reloading ?? 0, 1) * 100}%` }} />
          </View>
        </View>
      </Show>
      <Show when={props.magazine <= 0 && (props.reloading ?? -1) < 0}>
        <Text class="text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-[#ff4d5e] animate-pulse text-shadow-2xs text-stroke-3 text-stroke-bg">{t("shooter.ammo.empty")}</Text>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<AmmoCounterProps>> = {
  default: {"weapon":"VK-7 Carbine","mode":"auto","magazine":24,"capacity":30,"reserve":120,"reloading":-1,"low":0.25,"secondary":"Hollow SMG","secondaryAmmo":32,"secondaryKey":"2"},
  low: {"weapon":"Brace-12","mode":"single","magazine":2,"capacity":6,"reserve":18,"reloading":-1,"low":0.34,"secondary":"","secondaryAmmo":-1,"secondaryKey":""},
  empty: {"weapon":"Talon AR","mode":"burst","magazine":0,"capacity":30,"reserve":0,"reloading":-1,"low":0.25,"secondary":"Wren Pistol","secondaryAmmo":15,"secondaryKey":"2"},
  reloading: {"weapon":"Wren Revolver","mode":"single","magazine":0,"capacity":6,"reserve":42,"reloading":0.55,"low":0.34,"secondary":"","secondaryAmmo":-1,"secondaryKey":""},
  belt: {"weapon":"Rampart LMG","mode":"auto","magazine":148,"capacity":200,"reserve":400,"reloading":-1,"low":0.25,"secondary":"","secondaryAmmo":-1,"secondaryKey":""},
  overflow: {"weapon":"Northern Relay Extended Marksman Rifle Mark Seven","mode":"burst","magazine":1024,"capacity":1024,"reserve":123456,"reloading":-1,"low":0.25,"secondary":"Hollow-Point Compact Submachine Gun","secondaryAmmo":12345,"secondaryKey":"Left Shoulder"},
};
