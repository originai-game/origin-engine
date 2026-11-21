import type { Texture } from "@origingame/engine";
import {Image, Text, packClass, type UiEvent, View} from "@origingame/engine/ui";
import { Button } from "../controls/button";

export interface CardProps {
  /** Its heading. */
  title: string;
  /** A line or two under the heading; empty: none. */
  body: string;
  /** A picture the game registered across its top; 0: none. */
  image: Texture | 0;
  /** A badge at its top end (New, -20%); empty: none. */
  badge: string;
  /** Content the caller supplies (was the actions snippet). */
  actions?: (() => JSX.Element) | undefined;
  /** The open event, with the row's key where it comes from a row. */
  onOpen?: (e: UiEvent, key?: string) => void;
}

export function Card(props: CardProps) {
  return (
    <View class={`text-fg w-70 max-w-full p-0 gap-2 disabled:text-fg-subtle ${packClass("Panel", "frame")}`}>
      <Button class={`text-fg bg-[#171f30d6] border border-[#ffffff2e] nine-plate-cut w-full text-start p-3 ${packClass("Button", "plate")}`} onPress={(e) => props.onOpen?.(e)} label={props.title}>
        <View class="w-full gap-2">
          <Image class={`w-full aspect-[16/9] nine-plate-cut data-hidden:hidden`} src={props.image} data-hidden={props.image === 0} />
          <View class="flex-wrap items-start gap-y-1 gap-x-2">
            <Text class={`font-display text-[1.143rem] font-bold leading-tight tracking-[-0.01em] text-fg grow shrink basis-30 text-start whitespace-normal m-0`}>
              {props.title}
            </Text>
            <View class="text-[0.857rem] font-bold tracking-[0.14em] uppercase data-hidden:hidden" data-hidden={props.badge === ""}>
              <Text class="whitespace-normal">{props.badge}</Text>
            </View>
          </View>
          <Text class={`text-fg/66 text-start whitespace-normal m-0 data-hidden:hidden`} data-hidden={props.body === ""}>
            {props.body}
          </Text>
        </View>
      </Button>
      <View class="flex-row flex-wrap items-center justify-end gap-2">{props.actions?.()}</View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CardProps>> = {
  default: {"title":"Ember Blade","body":"Burns on hit. Two-handed.","image":0,"badge":"New"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"open":"Open"} as const;
