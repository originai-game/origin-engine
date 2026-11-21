import { Show, View, t } from "@origingame/engine/ui";
import "./rpg.en.ogt";

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
    <View class={`relative size-[6.171rem] shrink-0 pointer-events-none compact:size-[4.114rem]`} role="status" label={props.target !== "" ? t("rpg.lock.target", { name: props.target }) : t("rpg.lock.none")} data-locked={props.target !== ""}>
      <Show when={props.target !== ""}>
        {/* The ring: the conic arcs cut from a disc by a mask stand in as
            one lit band with the accent glow. */}
        <View class="absolute inset-0 rounded-full shadow-[inset_0px_0px_0px_2.288px_#ffffff,0px_0px_6px_1px_#38e8ffb3] animate-pop" />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgLockOnProps>> = {
  default: {"target":"Ashen Colossus"},
  empty: {"target":""},
};
