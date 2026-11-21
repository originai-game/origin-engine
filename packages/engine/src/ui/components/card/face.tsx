import type { Texture } from "@origingame/engine";
import { Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import "./card.en.ogt";

export interface CardFaceProps {
  /** The card's name as the game words it (an upgraded card's name included). */
  name: string;
  /** Its cost as the game words it (2, X, 0); empty hides the cost gem (a relic, an unplayable curse). */
  cost: string;
  /** Its art as an icon in the style's or the game's atlas, used when art is 0. */
  icon: string;
  /** A painted picture (og.load.texture) filling the art window; 0 shows the icon. */
  art: Texture | 0;
  /** The card's type for the pack's frame (attack, skill, power, curse, status, or the game's own word); it reaches the frame as data-type. */
  type: string;
  /** The type line as the game words it (Attack, Skill · Rare); empty hides it. */
  typeLabel: string;
  /** common, uncommon, rare, epic, legendary or mythic; rare and above draw the ornate card frame. */
  rarity: string;
  /** The rules text as the game words it, every number already computed; it wraps, up to four lines and then an ellipsis, and the art window gives it the room. */
  text: string;
  /** Dims the art and the frame (a card that cannot be played or is not owned); the words keep their ink. */
  dim?: boolean;
  /** How short its place is: empty is full height; medium, small and tiny clamp the rules text to 3, 2 and 1 lines (tiny also drops the type line). */
  size?: string;
}

/**
 * One card's face: the cost gem, the art window over its rarity's glow,
 * the name on its plate, the type line and the rules text. It draws what
 * the game words and reports nothing; the hand, the collection and the
 * reward wrap it in their buttons. It fills the box it is given (give the
 * box the card's aspect ratio). The frame wears its type's colour; the
 * name takes two lines and the rules text four, each ending in an
 * ellipsis at a whole line, and the art window yields its height to them.
 */
export function CardFace(props: CardFaceProps) {
  return (
    <View class={`group/cardface nine-plate-cut bg-glass text-rim-strong grow size-full p-1 gap-0.5 overflow-hidden data-type-attack:text-danger data-type-skill:text-info data-type-power:text-epic data-type-curse:text-fg-muted data-type-status:text-fg-muted data-type-relic:text-gold data-dim:text-rim`} data-framed={props.rarity === "rare" || props.rarity === "epic" || props.rarity === "legendary"} data-type-attack={(props.type) === 'attack'} data-type-curse={(props.type) === 'curse'} data-type-power={(props.type) === 'power'} data-type-relic={(props.type) === 'relic'} data-type-skill={(props.type) === 'skill'} data-type-status={(props.type) === 'status'} data-dim={props.dim === true} data-medium={props.size === 'medium' || props.size === 'small' || props.size === 'tiny'} data-small={props.size === 'small' || props.size === 'tiny'} data-tiny={props.size === 'tiny'}>
      <View class="absolute bg-linear-to-b from-current to-transparent to-32% opacity-22 inset-0" />
      <View class={`flex-row items-center grow shrink basis-0 min-h-[18%] justify-center nine-plate-cut bg-tile-pressed text-border-strong gap-0 overflow-hidden`}>
        <View class="absolute opacity-45 inset-0" />
        <Show when={props.art} fallback={
            <Icon class="w-[56%] h-[76%] text-fg shrink" name={props.icon} />
        }>
          <Image class="absolute object-cover inset-0 size-full" src={props.art} />
        </Show>
      </View>
      <Text class={`text-fg text-center line-clamp-2 text-base font-bold leading-[1.15] ${packClass("Plate", "frame")} py-0.5 px-1 m-0 overflow-hidden group-data-medium/cardface:line-clamp-1`}>
        {props.name}
      </Text>
      <Show when={props.typeLabel}>
        <Text class={`text-center whitespace-nowrap text-ellipsis text-[0.786rem] font-bold tracking-[0.14em] uppercase text-fg-muted m-0 overflow-hidden group-data-type-attack/cardface:text-danger group-data-type-skill/cardface:text-info group-data-type-power/cardface:text-epic group-data-type-curse/cardface:text-fg-muted group-data-tiny/cardface:hidden`}>
          {props.typeLabel}
        </Text>
      </Show>
      <Show when={props.text}>
        <Text class={`text-center line-clamp-4 text-[0.893rem] leading-snug text-fg bg-tile-pressed py-0.5 px-1 m-0 overflow-hidden group-data-medium/cardface:line-clamp-3 group-data-small/cardface:line-clamp-2 group-data-tiny/cardface:line-clamp-1`}>
          {props.text}
        </Text>
      </Show>
      <View class="absolute inset-0 pointer-events-none hidden group-data-framed/cardface:flex nine-card-frame" />
      <Show when={props.cost}>
        <View class={`flex-row items-center absolute left-0 top-0 w-[2.143rem] h-[2.4rem] justify-center bg-linear-to-b from-hud-accent-hi to-hud-accent text-hud-accent-fg text-[1.286rem] font-bold gap-0`} label={t("card.face.cost", { cost: props.cost })}>
          <Text class="whitespace-nowrap shrink">{props.cost}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<CardFaceProps>> = {
  default: {"name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage. Apply 1 Burn.","dim":false},
  relic: {"name":"Lantern of Dusk","cost":"","icon":"crown","art":0,"type":"relic","typeLabel":"Relic","rarity":"rare","text":"At the start of each combat, gain 1 energy.","dim":false},
  overflow: {"name":"Thousandfold Lamentation of the Drowned Cathedral","cost":"10","icon":"skull","art":0,"type":"power","typeLabel":"Power · Legendary","rarity":"legendary","text":"At the start of your turn, every enemy loses 3 Strength, you draw 2 cards, gain 2 Block for each card in your discard pile and Exhaust a random card from your hand. Ethereal. Innate. Retain.","dim":false},
  dim: {"name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power · Rare","rarity":"rare","text":"Draw 2 more cards each turn.","dim":true},
};
