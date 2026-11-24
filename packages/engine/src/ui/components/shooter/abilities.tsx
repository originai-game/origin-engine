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
 * every cooldown and charge; read only. Every part is a pack slot
 * (`Ability.frame`, `.face`, `.ring`, `.sweep`, `.count`, `.chip.*`), so a
 * theme sets its size, shape, ring, glow, sweep and type.
 */
export function ShooterAbilities(props: ShooterAbilitiesProps) {
  return (
    <View class="flex-row items-end gap-x-3.5 max-w-full compact:gap-x-2 text-fg" label={t("shooter.abilities.name")}>
      <For each={props.slots ?? []} key={(s) => s.id}>
        {(s) => {
          const ult = () => s().ultimate === true;
          const charge = () => Math.min(Math.max(s().charge ?? 0, 0), 1);
          const ready = () => (s().ready ?? false) || (ult() && charge() >= 1);
          const cooling = () => (s().cooldown ?? 0) > 0;
          return (
            <View class="flex-col items-center gap-y-1.25 shrink basis-auto min-w-0 data-ultimate:gap-y-3" label={s().name} data-ultimate={ult()}>
              <View
                class={`relative ${packClass("Ability", "box")} ${ult() ? packClass("Ability", "box", "ultimate") : ""} ${ult() ? packClass("Ability", "frame", "ultimate") : packClass("Ability", "frame")} ${ready() && !ult() ? packClass("Ability", "frame", "ready") : ""} ${s().disabled ? packClass("Ability", "frame", "disabled") : ""}`}
              >
                <Show when={ult()}>
                  <View
                    class={`absolute inset-0 ${packClass("Ability", "ring")} ${ready() ? packClass("Ability", "ring", "ready") : ""}`}
                    role="meter"
                    label={t("shooter.abilities.charge")}
                    valueMin={0}
                    valueMax={100}
                    valueNow={charge() * 100}
                  >
                    <View class={`absolute inset-0 ${packClass("Ability", "ring", "fill")}`} wipe={charge()} />
                  </View>
                </Show>
                <View class={`absolute ${packClass("Ability", "face")} ${ult() ? packClass("Ability", "face", "ultimate") : ""}`}>
                  <Show when={s().icon} fallback={
                    <View class="size-full flex-row items-center justify-center">
                      <Show when={s().glyph}>
                        <Icon class="size-1/2 text-fg" name={s().glyph} />
                      </Show>
                    </View>
                  }>
                    <Image class="size-full" src={s().icon ?? 0} />
                  </Show>
                  <Show when={cooling()}>
                    <View
                      class={`absolute inset-0 ${packClass("Ability", "sweep")} ${ult() ? packClass("Ability", "sweep", "ultimate") : ""}`}
                      wipe={Math.min(s().cooldown ?? 0, 1)}
                      wipeCcw
                    />
                  </Show>
                </View>
                <Show when={cooling() && (s().time ?? "") != ""}>
                  <View class="absolute inset-0 flex-row items-center justify-center pointer-events-none">
                    <Text class={`whitespace-nowrap ${packClass("Ability", "count")}`}>{s().time}</Text>
                  </View>
                </Show>
                <Show when={ult() && !cooling()}>
                  <View class="absolute inset-x-0 -bottom-2 flex-row justify-center pointer-events-none">
                    <Show when={charge() < 1} fallback={
                      <Text class={`whitespace-nowrap ${packClass("Ability", "chip", "ready")}`}>{t("shooter.abilities.ready")}</Text>
                    }>
                      <Text class={`whitespace-nowrap ${packClass("Ability", "chip", "ult")}`}>
                        {`${formatNumber(charge() * 100, { decimals: 0 })}%`}
                      </Text>
                    </Show>
                  </View>
                </Show>
                <Show when={(s().count ?? 0) > 0}>
                  <Text class={`absolute -top-1.5 -right-1.5 max-w-15.5 whitespace-nowrap truncate ${packClass("Ability", "chip", "charges")}`}>
                    {t("shooter.abilities.count", { count: formatNumber(s().count ?? 0) })}
                  </Text>
                </Show>
              </View>
              <Show when={(s().key ?? "") != ""}>
                <Text class={`shrink basis-auto max-w-full min-w-5.5 whitespace-nowrap truncate pointer-coarse:hidden ${packClass("Ability", "chip", "key")}`}>
                  {s().key}
                </Text>
              </Show>
            </View>
          );
        }}
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
