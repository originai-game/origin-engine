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
    <View class="flex-row items-end gap-x-3.5 max-w-full compact:gap-x-2 text-fg" label={t("shooter.abilities.name")}>
      <For each={props.slots ?? []} key={(s) => s.id}>
        {(s) => (
          <View
            class={`flex-col items-center gap-y-1.25 shrink basis-auto min-w-0 group/ability data-ultimate:size-21 compact:data-ultimate:size-11`}
            label={s().name}
            data-ultimate={s().ultimate === true}
            data-ready={s().ready || (s().ultimate && (s().charge ?? 0) >= 1)}
            data-disabled={s().disabled === true}
          >
            <View class="relative size-13 compact:size-9 group-data-ultimate/ability:size-21 compact:group-data-ultimate/ability:size-11">
              <Show when={s().ultimate}>
                <View
                  class="absolute inset-0 rounded-full overflow-hidden border-3 border-rim bg-tile"
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
                class={`absolute inset-0 flex-row items-center justify-center group-data-ultimate/ability:inset-11/100`}
              >
                <View
                  class={`relative size-full rounded-card bg-linear-to-b from-rim-strong via-rim to-rim-soft group-data-ready/ability:from-hud-accent-hi group-data-ready/ability:to-hud-accent group-data-ready/ability:shadow-glow group-data-disabled/ability:opacity-45 group-data-ultimate/ability:rounded-tl-control group-data-ultimate/ability:rounded-tr-control`}
                >
                  <View class="absolute inset-0.5 rounded-block overflow-hidden bg-plate-hi">
                    <Show when={s().icon} fallback={
                      <View class="size-full flex-row items-center justify-center">
                        <Show when={s().glyph}>
                          <Icon class="size-1/2 text-fg" name={s().glyph} />
                        </Show>
                      </View>
                    }>
                      <Image class="size-full" src={s().icon ?? 0} />
                    </Show>
                    <Show when={(s().cooldown ?? 0) > 0}>
                      <View
                        class="absolute left-0 right-0 top-0 bg-shade-deep"
                        style={{ height: `${Math.min(s().cooldown ?? 0, 1) * 100}%` }}
                      />
                    </Show>
                  </View>
                  <Show when={(s().cooldown ?? 0) > 0 && (s().time ?? "") != ""}>
                    <View class="absolute inset-0 flex-row items-center justify-center pointer-events-none">
                      <Text class="text-title leading-none font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-base group-data-ultimate/ability:text-base compact:group-data-ultimate/ability:text-caption">
                        {s().time}
                      </Text>
                    </View>
                  </Show>
                  <Show when={s().ultimate && (s().charge ?? 0) < 1}>
                    <View class="absolute inset-0 flex-row items-center justify-center pointer-events-none">
                      <Text class="text-title leading-none font-bold tabular-nums whitespace-nowrap text-fg text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-base group-data-ultimate/ability:text-base compact:group-data-ultimate/ability:text-caption">
                        {`${formatNumber(Math.max(s().charge ?? 0, 0) * 100, { decimals: 0 })}%`}
                      </Text>
                    </View>
                  </Show>
                </View>
              </View>
              <Show when={(s().count ?? 0) > 0}>
                <Text
                  class={`absolute -top-1.5 -right-1.5 max-w-15.5 py-0 px-1.25 text-xs font-bold tabular-nums whitespace-nowrap truncate text-fg nine-chip-cut bg-tile ${packClass("Chip", "frame")}`}
                >
                  {t("shooter.abilities.count", { count: formatNumber(s().count ?? 0) })}
                </Text>
              </Show>
            </View>
            <Show when={(s().key ?? "") != ""}>
              <Text
                class={`shrink basis-auto max-w-full min-w-5.5 py-0 px-1 text-xs leading-normal font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key whitespace-nowrap truncate pointer-coarse:hidden`}
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
