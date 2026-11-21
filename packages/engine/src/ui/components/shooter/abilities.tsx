import { For, Icon, Image, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./shooter.en.ogt";

export interface AbilitySlotData {
  id: string;
  /** What a reader calls it (Phase dash). */
  name: string;
  /** Its art, a texture (og.load.texture); 0 draws the glyph in its place. */
  icon?: Texture | 0;
  /** An icon of the atlas, drawn when there is no art; empty is none. */
  glyph?: string;
  /** The input as the game names it (Q), on a key cap under the slot; empty is none. */
  key?: string;
  /** The share of its cooldown still to run, 0 to 1: a dark sweep over the art. */
  cooldown?: number;
  /** The seconds left as the game words it (6); shown over the sweep. */
  time?: string;
  /** The charges or items held, on a tag (×2; 0 hides it). */
  count?: number;
  /** Lit (a lit edge and glow). */
  ready?: boolean;
  /** Greyed. */
  disabled?: boolean;
  /** The ultimate: larger, in a ring that fills with its charge. */
  ultimate?: boolean;
  /** The ultimate's charge, 0 to 1; its share in figures until it is ready. */
  charge?: number;
}

export interface ShooterAbilitiesProps {
  /** The abilities and items on the bar in the game's order. The game owns every cooldown and charge; read only. */
  slots?: ReadonlyArray<AbilitySlotData>;
}

/**
 * The ability bar at the foot of the screen: each ability in an octagon
 * with a lit edge, its cooldown a dark sweep over the art with the seconds
 * left, its charges on a tag and its key cap under it; the ultimate in a
 * larger hexagon inside a ring that fills with its charge. The game owns
 * every cooldown and charge; read only. Approximations: the octagon and
 * hexagon cuts are rounded; the cooldown's conic mask is a top-down veil;
 * the charge ring's SVG arc is a View stand-in that fills from its foot;
 * the ready glow and the grey-out filters are shadows and opacity.
 */
export function ShooterAbilities(props: ShooterAbilitiesProps) {
  return (
    <View class="flex-row items-end gap-x-[1rem] max-w-full compact:gap-x-[0.571rem] text-fg" label={t("shooter.abilities.name")}>
      <For each={props.slots ?? []} key={(s) => s.id}>
        {(s) => (
          <View
            class={`flex-col items-center gap-y-[0.357rem] shrink basis-auto min-w-0 group/ability data-ultimate:size-[6rem] compact:data-ultimate:size-[3.143rem]`}
            label={s().name}
            data-ultimate={s().ultimate === true}
            data-ready={s().ready || (s().ultimate && (s().charge ?? 0) >= 1)}
            data-disabled={s().disabled === true}
          >
            <View class="relative size-[3.714rem] compact:size-[2.571rem] group-data-ultimate/ability:size-[6rem] compact:group-data-ultimate/ability:size-[3.143rem]">
              <Show when={s().ultimate}>
                <View
                  class="absolute inset-0 rounded-full overflow-hidden border-[3px] border-white/14 bg-tile"
                  role="meter"
                  label={t("shooter.abilities.charge")}
                  valueMin={0}
                  valueMax={100}
                  valueNow={(s().charge ?? 0) <= 0 ? 0 : (s().charge ?? 0) >= 1 ? 100 : (s().charge ?? 0) * 100}
                >
                  <View
                    class="absolute inset-x-0 bottom-0 bg-hud-accent"
                    style={{ height: `${Math.min(Math.max(s().charge ?? 0, 0), 1) * 100}%` }}
                  />
                </View>
              </Show>
              <View
                class={`absolute inset-0 flex-row items-center justify-center group-data-ultimate/ability:inset-[11%]`}
              >
                <View
                  class={`relative size-full rounded-[0.714rem] bg-linear-to-b from-white/75 via-white/22 to-white/14 group-data-ready/ability:from-white group-data-ready/ability:to-hud-accent group-data-ready/ability:shadow-[0px_0px_0.714rem_rgb(56_232_255/0.55)] group-data-disabled/ability:opacity-45 group-data-ultimate/ability:rounded-tl-[0.357rem] group-data-ultimate/ability:rounded-tr-[0.357rem]`}
                >
                  <View class="absolute inset-[0.143rem] rounded-[0.571rem] overflow-hidden bg-plate-hi">
                    <Show when={s().icon} fallback={
                      <View class="size-full flex-row items-center justify-center">
                        <Show when={s().glyph}>
                          <Icon class="size-[50%] text-fg" name={s().glyph} />
                        </Show>
                      </View>
                    }>
                      <Image class="size-full" src={s().icon ?? 0} />
                    </Show>
                    <Show when={(s().cooldown ?? 0) > 0}>
                      <View
                        class="absolute left-0 right-0 top-0 bg-black/68"
                        style={{ height: `${Math.min(s().cooldown ?? 0, 1) * 100}%` }}
                      />
                    </Show>
                  </View>
                  <Show when={(s().cooldown ?? 0) > 0 && (s().time ?? "") != ""}>
                    <View class="absolute inset-0 flex-row items-center justify-center pointer-events-none">
                      <Text class="text-[1.714rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-[1.143rem] group-data-ultimate/ability:text-[1.143rem] compact:group-data-ultimate/ability:text-[0.929rem]">
                        {s().time}
                      </Text>
                    </View>
                  </Show>
                  <Show when={s().ultimate && (s().charge ?? 0) < 1}>
                    <View class="absolute inset-0 flex-row items-center justify-center pointer-events-none">
                      <Text class="text-[1.714rem] leading-none font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-[1.143rem] group-data-ultimate/ability:text-[1.143rem] compact:group-data-ultimate/ability:text-[0.929rem]">
                        {`${formatNumber(Math.max(s().charge ?? 0, 0) * 100, { decimals: 0 })}%`}
                      </Text>
                    </View>
                  </Show>
                </View>
              </View>
              <Show when={(s().count ?? 0) > 0}>
                <Text
                  class={`absolute -top-[0.429rem] -right-[0.429rem] max-w-[4.457rem] py-0 px-[0.357rem] text-[0.857rem] font-bold tabular-nums whitespace-nowrap truncate text-fg nine-chip-cut bg-tile ${packClass("Chip", "frame")}`}
                >
                  {t("shooter.abilities.count", { count: formatNumber(s().count ?? 0) })}
                </Text>
              </Show>
            </View>
            <Show when={(s().key ?? "") != ""}>
              <Text
                class={`shrink basis-auto max-w-full min-w-[1.571rem] py-0 px-1 text-[0.857rem] leading-[1.5] font-bold text-center text-key-fg bg-linear-to-b from-white to-key-hi rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] whitespace-nowrap truncate pointer-coarse:hidden`}
              >
                {s().key}
              </Text>
            </Show>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterAbilitiesProps>> = {
  default: {"slots":[{"id":"dash","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":0.38,"time":"6","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"frag","name":"Frag grenade","icon":0,"glyph":"bomb","key":"G","cooldown":0,"time":"","count":2,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0.74},{"id":"stim","name":"Stim","icon":0,"glyph":"potion","key":"4","cooldown":0,"time":"","count":4,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"wall","name":"Barrier","icon":0,"glyph":"shield","key":"X","cooldown":0.92,"time":"38","count":0,"ready":false,"disabled":true,"ultimate":false,"charge":0}]},
  ready: {"slots":[{"id":"q","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":0,"time":"","count":0,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":true,"disabled":false,"ultimate":true,"charge":1}]},
  cooling: {"slots":[{"id":"q","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":1,"time":"12","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"e","name":"Scan","icon":0,"glyph":"map","key":"E","cooldown":0.1,"time":"1","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0}]},
  overflow: {"slots":[{"id":"a","name":"A","icon":0,"glyph":"bolt","key":"Left Shoulder","cooldown":0.5,"time":"1:02:44","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"b","name":"B","icon":0,"glyph":"bomb","key":"Right Trigger","cooldown":0,"time":"","count":12345,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"C","icon":0,"glyph":"star","key":"Left Stick + Right Stick","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0.999},{"id":"d","name":"D","icon":0,"glyph":"potion","key":"Dpad Up","cooldown":0,"time":"","count":99,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"e","name":"E","icon":0,"glyph":"shield","key":"Y","cooldown":0.2,"time":"8","count":0,"ready":false,"disabled":true,"ultimate":false,"charge":0},{"id":"f","name":"F","icon":0,"glyph":"map","key":"B","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0}]},
};
