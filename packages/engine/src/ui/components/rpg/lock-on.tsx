import { Show, View, t } from "@origingame/engine/ui";
import "./rpg.en.strings.ts";

export interface RpgLockOnProps {
  /** The locked target's name for a reader; empty hides the reticle (nothing is locked). */
  target: string;
}

/**
 * The lock-on reticle the page centres on the locked target: lit arcs
 * round it with a glow in the HUD's accent. The game decides what is
 * locked and where it stands on screen; read only.
 */
export function RpgLockOn(props: RpgLockOnProps) {
  return (
    <View class={`relative size-21.5 shrink-0 pointer-events-none compact:size-14.5`} role="status" label={props.target !== "" ? t("rpg.lock.target", { name: props.target }) : t("rpg.lock.none")} data-locked={props.target !== ""}>
      <Show when={props.target !== ""}>
        {/* The reticle: the svelte's four conic arcs (white, 40° each at
            0°, 70°, 160° and 250°, clockwise from up) as nine-ring Views;
            the accent glow went with the mask. */}
        <View class={`absolute inset-0 animate-pop`}>
          <View class={`absolute inset-0 nine-ring bg-fg`} wipe={1 / 9} wipeStart={0} />
          <View class={`absolute inset-0 nine-ring bg-fg`} wipe={1 / 9} wipeStart={1.2217} />
          <View class={`absolute inset-0 nine-ring bg-fg`} wipe={1 / 9} wipeStart={2.7925} />
          <View class={`absolute inset-0 nine-ring bg-fg`} wipe={1 / 9} wipeStart={4.3633} />
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgLockOnProps>> = {
  default: {"target":"Ashen Colossus"},
  empty: {"target":""},
};
