import { View, t } from "@origingame/engine/ui";
import { ActionHints, RpgQuestTracker } from "@origingame/engine/ui/components";
import "../../content/ui/hud.en.ts";

export interface HudProps {
  /** How far the lighthouse is, as the game words it (64 m). */
  distance: string;
}

const HINTS = [
  { action: "move", hint: "Walk" }, { action: "look", hint: "Look" },
  { action: "weather", hint: "Weather" }, { action: "season", hint: "Season" }, { action: "time", hint: "Time of day" },
];

/** The wanderer's HUD: the objective top right, the controls along the bottom. */
export function Hud(props: HudProps) {
  return (
    <View class="absolute inset-0 justify-between pt-[calc(env(hud-inset-top)+16px)] pr-[calc(env(hud-inset-right)+16px)] pb-[calc(env(hud-inset-bottom)+16px)] pl-[calc(env(hud-inset-left)+16px)] compact:p-2">
      <View class="flex-row justify-end">
        <View class="w-76 compact:w-full">
          <RpgQuestTracker label="" name="The Last Light" distance={props.distance} steps={[
            { id: "path", text: "Follow the cliff path", done: false, tag: "" },
            { id: "lamp", text: "Reach the lighthouse", done: false, tag: "" },
          ]} />
        </View>
      </View>
      <View class="items-center compact:hidden">
        <ActionHints hints={HINTS} label={t("hud.controls")} />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudProps>> = {
  default: { distance: "64 m" },
};
