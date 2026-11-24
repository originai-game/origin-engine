import type { Texture } from "@origingame/engine";
import { Icon, Image, packClass, Show, t, Text, View, type UiEvent } from "@origingame/engine/ui";
import "./rpg.en.ogt";

export interface RpgPlaceCardProps {
  /** The place's name. */
  name: string;
  /** A line or two about it; empty hides it. */
  about: string;
  /** Its picture (og.load.texture); 0 hides it. */
  picture: Texture | 0;
  /** The player has marked it: the button offers Unmark. */
  marked: boolean;
  /** Fast travel to it is possible: the button is enabled. */
  canTravel: boolean;
  /** Mark or unmark. */
  onMark?: (e: UiEvent) => void;
  /** Fast travel. */
  onTravel?: (e: UiEvent) => void;
}

/**
 * A place picked on the map, on dark glass: its picture at the start
 * fading into the glass, its name in the HUD's title type over a gold
 * rule, a line about it, and two buttons, Mark (a lit ghost plate) and
 * Fast travel (the gold primary plate). Reports mark (marking or
 * unmarking) and travel (fast travel). Narrow the line about it is left
 * out, so the name and the buttons keep their room.
 */
export function RpgPlaceCard(props: RpgPlaceCardProps) {
  return (
    <View class={`flex-row items-stretch w-full min-w-0 bg-glass-lo ${packClass("Panel", "frame")}`} role="region" label={props.name}>
      <Show when={props.picture}>
        <View class="relative shrink-0 w-17/50 compact:w-13/50 overflow-hidden">
          <Image class="absolute inset-0 size-full" src={props.picture} />
          <View class="absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-glass-lo to-transparent" />
        </View>
      </Show>
      <View class="flex-col justify-center min-w-0 grow gap-y-1 py-3 px-4 compact:py-2 compact:px-3">
        <Text role="heading" level={3} class={`max-w-full text-left text-display-lg leading-flat font-bold whitespace-nowrap overflow-hidden text-fg compact:text-3xl`}>
          {props.name}
        </Text>
        <View class="w-full h-0.5 shrink-0 bg-linear-to-r from-gold to-transparent" />
        <Show when={props.about}>
          <Text class={`line-clamp-2 text-left text-base leading-normal text-fg-muted overflow-hidden compact:hidden`}>
            {props.about}
          </Text>
        </Show>
        <View class="flex-row w-full shrink-0 gap-1 pt-1">
          <View class={`transition duration-fast flex-1 min-w-0 min-h-11 flex-row items-center justify-center gap-1 px-2 compact:px-1 text-caption font-bold tracking-label uppercase text-fg data-marked:text-gold ${packClass("Button", "plate")}`} role="button" label={props.marked ? t("rpg.place.unmark") : t("rpg.place.mark")} pressed={props.marked} selected={props.marked} data-marked={props.marked} onPress={(e) => props.onMark?.(e)}>
            <Icon class="shrink-0 size-3.75 compact:hidden" name="flag" />
            <Text class="whitespace-nowrap">{props.marked ? t("rpg.place.unmark") : t("rpg.place.mark")}</Text>
          </View>
          <View class={`transition duration-fast flex-1 min-w-0 min-h-11 flex-row items-center justify-center gap-1 px-2 compact:px-1 text-caption font-bold tracking-label uppercase disabled:opacity-dim ${packClass("Button", "plate", "primary")}`} role="button" label={t("rpg.place.travel")} disabled={!props.canTravel} onPress={(e) => props.onTravel?.(e)}>
            <Icon class="shrink-0 size-3.75 compact:hidden" name="map" />
            <Text class="whitespace-nowrap">{t("rpg.place.travel")}</Text>
          </View>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgPlaceCardProps>> = {
  default: {"name":"Dawnhold Ruins","about":"Crumbling walls overlook the valley.","picture":0,"marked":false,"canTravel":true},
  marked: {"name":"Dawnhold Ruins","about":"Crumbling walls overlook the valley. The old bell still rings at dusk, though no one has climbed the tower in forty years.","picture":0,"marked":true,"canTravel":false},
};
