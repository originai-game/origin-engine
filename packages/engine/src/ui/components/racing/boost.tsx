import type { Texture } from "@origingame/engine";
import { For, Image, Show, Text, View, t } from "@origingame/engine/ui";
import "./racing.en.ogt";

export interface RacingBoostProps {
  /** The meter's name as the game words it (Nitro, ERS, Boost); empty takes the kit's (Boost). */
  label?: string;
  /** The meter's picture (og.load.texture) in a cut tile before it; 0 hides it. */
  icon: Texture | 0;
  /** The input that fires it as the game names it (Shift), on a key cap after the label; empty for none; hidden under a touch screen. */
  key: string;
  /** What is left now. */
  value: number;
  /** The most there can be. Default 1. */
  max?: number;
  /** The cells the bar is cut into (5); 1 draws one bar. Default 5. */
  cells?: number;
  /** It is firing now: the fill glows (data-firing). */
  firing: boolean;
}

/**
 * A boost meter (nitro, ERS) beside the tachometer: its picture in a cut
 * tile, the name with the key that fires it, and a leaning bar cut into
 * cells, glowing while it fires. The game owns the store and when it
 * fires; read only.
 */
export function RacingBoost(props: RacingBoostProps) {
  const label = () => ((props.label ?? "") !== "" ? (props.label ?? "") : t("racing.boost.name"));
  const frac = () => {
    const max = props.max ?? 1;
    return max <= 0 || props.value <= 0 ? 0 : props.value >= max ? 1 : props.value / max;
  };
  return (
    <View
      class={`group/boost flex-row items-center gap-x-2.5 w-60 max-w-full text-fg compact:w-full compact:gap-x-1.5`}
      role="meter"
      label={label()}
      valueMin={0}
      valueMax={(props.max ?? 1) > 0 ? (props.max ?? 1) : 1}
      valueNow={Math.max(props.value, 0)}
      data-firing={props.firing}
    >
      <Show when={props.icon}>
        <Image src={props.icon} class={`size-8.5 shrink-0 nine-plate-cut compact:size-6`} />
      </Show>
      <View class={`flex-1 min-w-0 flex-col gap-y-1.25`}>
        <View class="flex-row items-center gap-x-2 min-w-0">
          <Text class={`shrink min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap overflow-hidden text-ally-ink text-stroke-3 text-stroke-bg compact:tracking-normal`}>{label()}</Text>
          <Show when={(props.key ?? "") !== ""}>
            <Text class={`shrink min-w-5.5 max-w-3/5 pointer-coarse:hidden bg-linear-to-b from-key-hi to-key-lo text-hud-accent-fg rounded-control shadow-key text-xs leading-normal font-bold tabular-nums text-center whitespace-nowrap overflow-hidden py-0 px-1`}>{props.key}</Text>
          </Show>
        </View>
        <View class={`relative w-full h-3 overflow-hidden bg-well shadow-plate ring ring-rim compact:h-2`}>
          <View class="absolute left-0 top-0 h-full bg-ally group-data-firing/boost:shadow-glow group-data-firing/boost:shadow-ally" style={{ width: `${frac() * 100}%` }} />
          {/* The cell cuts (a repeating-gradient mask in the svelte): one separator per cell. */}
          <For each={Array.from({ length: Math.max(1, Math.floor(props.cells ?? 5)) - 1 })}>
            {(_, i) => (
              <View
                class="absolute h-full w-0.75 bg-tile pointer-events-none"
                style={{ left: `${((i() + 1) / Math.max(1, Math.floor(props.cells ?? 5))) * 100}%` }}
              />
            )}
          </For>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RacingBoostProps>> = {
  default: { "label": "Nitro", "icon": 0, "key": "Shift", "value": 60, "max": 100, "cells": 5, "firing": false },
  firing: { "label": "Nitro", "icon": 0, "key": "Shift", "value": 92, "max": 100, "cells": 5, "firing": true },
  empty: { "label": "ERS", "icon": 0, "key": "", "value": 0, "max": 100, "cells": 1, "firing": false },
  overflow: { "label": "Energy Recovery System Deployment", "icon": 0, "key": "Left Shoulder + Right Trigger", "value": 250, "max": 100, "cells": 40, "firing": true },
};
