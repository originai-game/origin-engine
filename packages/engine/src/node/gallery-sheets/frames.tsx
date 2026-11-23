// The UI gallery's frame sheet: the pack's nine-slice frame art (templates/tools/make-frame-art.py
// draws the default pack's and scifi's) by name, at the sizes a phone screen draws it: panels,
// plates, buttons in their states, slots with a rarity rim, chips, badges, a field and a divider.
// Not a component: the components take the art through their slots' pack dress.
import { Icon, Text, View, packClass } from "@origingame/engine/ui";

export interface FrameSheetProps {
  /** Which pieces: the containers (panel, plate, field, divider) or the controls (buttons, slots, chips, badges). */
  set: "containers" | "controls";
}

function Label(props: { text: string }) {
  return <Text class="self-start text-xs text-fg-muted">{props.text}</Text>;
}

function Containers() {
  return (
    <View class="flex flex-col gap-3">
      <Label text="nine-panel · nine-slot · nine-divider" />
      <View class={`flex flex-col gap-2 px-5 py-5 nine-panel ${packClass("Panel", "frame")}`}>
        <Text class="text-lg font-bold text-fg">Daily rewards</Text>
        <View class="h-2 self-stretch nine-divider" />
        <Text class="text-sm text-fg">Log in every day to collect.</Text>
        <View class="flex flex-row gap-2">
          {["coin", "gem", "chest", "energy"].map((name) => (
            <View class="size-14 items-center justify-center nine-slot"><Icon class="size-8" name={name} /></View>
          ))}
        </View>
      </View>
      <Label text="nine-plate" />
      <View class="flex flex-row gap-2">
        <View class="flex-row items-center gap-1 pl-2 pr-4 h-9 nine-plate"><Icon class="size-6" name="coin" /><Text class="font-bold text-art-fg">12,480</Text></View>
        <View class="flex-row items-center gap-1 pl-2 pr-4 h-9 nine-plate"><Icon class="size-6" name="gem" /><Text class="font-bold text-art-fg">320</Text></View>
        <View class="flex-row items-center gap-1 pl-2 pr-4 h-9 nine-plate"><Icon class="size-6" name="energy" /><Text class="font-bold text-art-fg">60/60</Text></View>
      </View>
      <Label text="nine-field" />
      <View class="h-11 justify-center px-3 nine-field"><Text class="text-art-fg">Player name</Text></View>
      <Label text="a tall panel (the art stretched)" />
      <View class={`h-40 items-center justify-center nine-panel ${packClass("Panel", "frame")}`}><Text class="text-fg">Inventory</Text></View>
    </View>
  );
}

const BUTTONS: Array<[string, string, string]> = [
  ["button", "Continue", "text-art-button-fg"],
  ["button-primary", "Start", "text-art-primary-fg"],
  ["button-danger", "Delete", "text-art-fg"],
];

function Controls() {
  return (
    <View class="flex flex-col gap-3">
      <Label text="nine-button[-primary|-danger] · rest, -hover, -pressed" />
      {BUTTONS.map(([art, label, ink]) => (
        <View class="flex flex-row gap-2">
          {["", "-hover", "-pressed"].map((state) => (
            <View class={`h-12 w-24 items-center justify-center pb-1 nine-${art}${state}`}><Text class={`font-bold ${ink}`}>{label}</Text></View>
          ))}
        </View>
      ))}
      <View class="flex flex-row gap-2 self-stretch">
        <View class="h-12 w-24 items-center justify-center pb-1 nine-button-disabled"><Text class="font-bold text-art-fg/80">Locked</Text></View>
        <View class="h-14 flex-1 min-w-0 items-center justify-center pb-1 nine-button-primary"><Text class="text-lg font-bold text-art-primary-fg">Claim rewards</Text></View>
      </View>
      <Label text="nine-slot + nine-slot-rim, tinted by rarity" />
      <View class="flex flex-row gap-2">
        {([["sword", ""], ["shield", "bg-uncommon"], ["potion", "bg-rare"], ["crown", "bg-epic"], ["gem", "bg-legendary"]] as Array<[string, string]>).map(([name, rarity]) => (
          <View class="size-14 nine-slot">
            <View class={`size-14 items-center justify-center ${rarity ? `nine-slot-rim ${rarity}` : ""}`}><Icon class="size-9" name={name} /></View>
          </View>
        ))}
      </View>
      <Label text="nine-chip-bevel (tinted) · nine-badge" />
      <View class="flex flex-row items-center gap-2">
        <View class="h-7 px-3 justify-center nine-chip-bevel bg-rare"><Text class="text-xs font-bold text-art-fg">Rare</Text></View>
        <View class="h-7 px-3 justify-center nine-chip-bevel bg-epic"><Text class="text-xs font-bold text-art-fg">Epic</Text></View>
        <View class="h-7 px-3 justify-center nine-chip-bevel bg-legendary"><Text class="text-xs font-bold text-art-fg">Legendary</Text></View>
        <View class="h-6 min-w-6 px-1.5 items-center justify-center nine-badge"><Text class="text-xs font-bold text-art-fg">3</Text></View>
        <View class="h-6 min-w-6 px-1.5 items-center justify-center nine-badge"><Text class="text-xs font-bold text-art-fg">99+</Text></View>
      </View>
    </View>
  );
}

export function FrameSheet(props: FrameSheetProps) {
  return (
    <View class="scheme-dark bg-bg self-stretch flex-1 p-4">
      {props.set === "containers" ? <Containers /> : <Controls />}
    </View>
  );
}

export const fixtures: Record<string, FrameSheetProps> = {
  containers: { set: "containers" },
  controls: { set: "controls" },
};
