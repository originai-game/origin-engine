import { Show, View, t } from "@origingame/engine/ui";
import { ActionHints, HudTimer, Minimap, RpgQuestTracker, StatBar } from "@origingame/engine/ui/components";
import type { Texture } from "@origingame/engine";
import "./hud.en.ogt";

export interface HudProps {
  /** The HUD is drawn; H turns it off. */
  shown?: boolean;
  /** Seconds left to make the drop. */
  seconds: number;
  /** The delivery's whole time. */
  total: number;
  /** The parcel's name, on the tracker's plate. */
  parcel: string;
  /** The delivery's steps, keyed rows of { id, text, done }. */
  steps: ReadonlyArray<{ id: string; text: string; done: boolean }>;
  /** How far the drop is, as the game words it (29 m). */
  distance: string;
  /** The parcel's condition, 0 to 100. */
  condition: number;
  /** The district's map texture (0 while none). */
  map: Texture | 0;
  /** The minimap's markers, keyed rows of { id, x, y, tone, player }. */
  markers: ReadonlyArray<{ id: string; x: number; y: number; tone: string; player: boolean }>;
  /** The district's name, under the minimap. */
  district: string;
  /** The switches, keyed rows of { action, hint, hold }. */
  hints: ReadonlyArray<{ action: string; hint: string; hold: boolean }>;
}

/**
 * The courier's HUD: the district's minimap top left, the time left to
 * make the drop top centre, the parcel and its steps top right, and the
 * switches along the bottom, each 16 px inside the HUD insets. The lower
 * third is left to the street. Where the top does not fit in one row (a
 * phone held upright) the parcel goes under the map and the clock, and the
 * switches give way to the touch controls.
 */
export function Hud(props: HudProps) {
  return (
    <Show when={props.shown}>
      <View class="absolute inset-0 justify-between pt-[calc(env(hud-inset-top)+16px)] pr-[calc(env(hud-inset-right)+16px)] pb-[calc(env(hud-inset-bottom)+16px)] pl-[calc(env(hud-inset-left)+16px)] compact:p-2">
        <View class="flex-row flex-wrap items-start gap-3 compact:gap-2">
          <Minimap texture={props.map} u0={0} v0={0} u1={1} v1={1} size={168} markers={props.markers} label={props.district} />
          <View class="grow shrink basis-0 items-center">
            <HudTimer seconds={props.seconds} total={props.total} countdown={true} warnAt={30} paused={false} label={t("hud.drop")} />
          </View>
          <View class="shrink basis-70 w-70 gap-2 compact:basis-full compact:w-full">
            <RpgQuestTracker label="" name={props.parcel} steps={props.steps} distance={props.distance} />
            <StatBar value={props.condition} max={100} kind="stamina" label={t("hud.parcel")} showValue={true} valueText={`${props.condition} %`} />
          </View>
        </View>
        <View class="flex-row justify-center compact:hidden">
          <ActionHints hints={props.hints} label={t("hud.switches")} />
        </View>
      </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<HudProps>> = {
  default: {"shown":true,"seconds":142,"total":180,"parcel":"Parcel 7C · Fragile","steps":[{"id":"collect","text":"Collect at Lantern Row","done":true},{"id":"deliver","text":"Deliver to Gate Nine","done":false}],"distance":"29 m","condition":96,"map":0,"markers":[{"id":"me","x":0.52,"y":0.89,"tone":"neutral","player":true},{"id":"drop","x":0.5,"y":0.32,"tone":"accent","player":false}],"district":"Lantern Row","hints":[{"action":"weather","hint":"Weather","hold":false},{"action":"season","hint":"Season","hold":false}]},
};
