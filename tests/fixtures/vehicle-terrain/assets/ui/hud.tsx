import { Text, View } from "@origingame/engine/ui";

export interface HudProps {
  /** The car's speed, whole km/h. */
  kmh: number;
}

/** The speed, bottom right, 24 px inside the HUD insets, clear of the host's controls. */
export function Hud(props: HudProps) {
  return (
    <View class="absolute inset-0 justify-end items-end pr-[calc(env(hud-inset-right)+24px)] pb-[calc(env(hud-inset-bottom)+24px)] pointer-events-none">
      <View class="gap-3 py-2.5 px-4 bg-surface rounded-panel border border-rim" role="status">
        <Text class="text-display-sm font-mono text-fg whitespace-nowrap">{props.kmh}</Text>
        <Text class="text-meta text-fg-muted">km/h</Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudProps>> = {
  default: { kmh: 42 },
};
