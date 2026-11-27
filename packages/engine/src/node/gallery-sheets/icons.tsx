// The UI gallery's icon sheet: the engine's game icon set (templates/styles/shared/icons/game.ts)
// at the sizes games draw it, on the pack's dark and light plates. Not a component: the gallery
// copies it into its project with the set's names (./names.ts, written from game-names.ts).
import { Icon, Text, View, packClass } from "@origingame/engine/ui";
import { INK, TONED } from "./names";

export interface IconSheetProps {
  /** Which icons: the toned (hero) ones, the ink ones, or a few of both at every size. */
  set: "toned" | "ink" | "ramp";
  /** The sizes drawn, in px, each a band of the set on each of the plates. */
  sizes: number[];
  /** The plates each band is drawn on: the pack's dark ground, its light one. */
  plates: Array<"dark" | "light">;
}

const SIZE: Record<number, string> = { 16: "size-4", 24: "size-6", 32: "size-8", 48: "size-12", 64: "size-16" };
const RAMP = ["coin", "gem", "energy", "attack", "home"];

function Band(props: { names: string[]; size: number; light: boolean }) {
  return (
    <View class={`${props.light ? "scheme-light" : ""} bg-surface text-fg rounded-panel p-2 flex flex-row flex-wrap gap-2`}>
      {props.names.map((name) => <Icon class={SIZE[props.size] ?? "size-8"} name={name} />)}
    </View>
  );
}

export function IconSheet(props: IconSheetProps) {
  const names = props.set === "toned" ? TONED : props.set === "ink" ? INK : RAMP;
  return (
    <View class="flex flex-col gap-2">
      {props.sizes.map((size) => (
        <View class="flex flex-col gap-1">
          <Text class={`self-start text-xs text-fg rounded-control px-2 ${packClass("Plate", "frame")}`}>{`${props.set} · ${size} px`}</Text>
          {props.plates.map((plate) => <Band names={names} size={size} light={plate === "light"} />)}
        </View>
      ))}
    </View>
  );
}

export const fixtures: Record<string, IconSheetProps> = {
  toned48: { set: "toned", sizes: [48], plates: ["dark", "light"] },
  toned32and24: { set: "toned", sizes: [32, 24], plates: ["dark", "light"] },
  ramp: { set: "ramp", sizes: [16, 24, 32, 48, 64], plates: ["dark", "light"] },
  ink32: { set: "ink", sizes: [32], plates: ["dark"] },
  ink32light: { set: "ink", sizes: [32], plates: ["light"] },
  ink24: { set: "ink", sizes: [24], plates: ["dark", "light"] },
};
