// The UI gallery's effect sheet: each Flourish (components/hud/flourish.tsx, the engine's UI effect
// presets) played once over what it decorates, centred on a dark game backdrop, as a phone shows it.
// Not a component: the gallery records its plays.
import { Flourish } from "@origingame/engine/ui/components/hud/flourish";
import { Icon, Text, View } from "@origingame/engine/ui";
import { Button } from "@origingame/engine/ui/controls/button";

export interface EffectSheetProps {
  /** Which effect, and for reveal which rarity. */
  kind: "reward" | "levelUp" | "purchase" | "reveal" | "highlight";
  rarity?: string;
}

function Target(props: EffectSheetProps) {
  if (props.kind == "levelUp") return <View class="px-6 py-2 rounded-panel bg-glass"><Text class="text-display-md font-bold text-gold">LV 12</Text></View>;
  if (props.kind == "highlight") return <Button variant="primary" class="h-14 w-56" onPress={() => {}}><Text>Claim</Text></Button>;
  if (props.kind == "purchase") return <View class="flex-row items-center gap-1 pl-2 pr-4 h-10 rounded-full bg-glass"><Icon class="size-7" name="coin" /><Text class="text-lg font-bold text-fg">12,480</Text></View>;
  const name = props.kind == "reward" ? "chest" : props.rarity == "legendary" ? "crown" : props.rarity == "rare" ? "shield" : "gem";
  return <View class="size-24 items-center justify-center rounded-field bg-tile border-2 border-border-strong"><Icon class="size-16" name={name} /></View>;
}

export function EffectSheet(props: EffectSheetProps) {
  return (
    <View class="scheme-dark absolute inset-0 bg-linear-to-b from-surface to-bg items-center justify-center">
      <Flourish kind={props.kind} rarity={props.rarity} shots={[{ id: "1" }]}>
        <Target {...props} />
      </Flourish>
      {/* The gallery's caption sits on the pack's ground, clear of the effect's light. */}
      <View class="absolute bottom-0 inset-x-0 h-10 bg-bg" />
      <View class="absolute bottom-0 inset-x-0 h-10 bg-scrim" />
      <Text class="absolute bottom-16 text-sm text-fg-muted">{`Flourish · ${props.kind}${props.rarity ? ` · ${props.rarity}` : ""}`}</Text>
    </View>
  );
}

export const fixtures: Record<string, EffectSheetProps> = {
  reward: { kind: "reward" },
  levelUp: { kind: "levelUp" },
  purchase: { kind: "purchase" },
  revealRare: { kind: "reveal", rarity: "rare" },
  revealEpic: { kind: "reveal", rarity: "epic" },
  revealLegendary: { kind: "reveal", rarity: "legendary" },
  highlight: { kind: "highlight" },
};
