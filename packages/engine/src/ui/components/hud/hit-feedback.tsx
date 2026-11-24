import { For, Show, View, packClass } from "@origingame/engine/ui";
import { DamageDirection, type DamageDirectionHit } from "./damage-direction";

export interface HitFeedbackProps {
  /** The latest hit's id: each new id flashes the screen's edges once. Empty is none. */
  flash?: string;
  /** The player is critical, as the game decides: the edges pulse until it clears. Default false. */
  critical?: boolean;
  /** Recent hits, each drawn as an arc toward where it came from (DamageDirection). The game removes a hit when its mark should go. */
  hits?: ReadonlyArray<DamageDirectionHit>;
}

/**
 * What a hit feels like on the HUD, opt in: the screen's edges flash as a
 * hit lands, pulse while the game says the player is critical, and an arc
 * points to where each recent hit came from. Every look and timing is the
 * theme's (Hit.vignette, Hit.vignette.critical, Hit.ring, Hit.arc and the
 * duration-hit and duration-arc tokens). It decides nothing: the game
 * names the hits and when the player is critical. Read only.
 */
export function HitFeedback(props: HitFeedbackProps) {
  return (
    <View class="absolute inset-0 pointer-events-none">
      <Show when={props.critical}>
        <View class={`absolute inset-0 ${packClass("Hit", "vignette", "critical")}`} />
      </Show>
      <For each={props.flash ? [props.flash] : []} key={(id) => id}>
        {() => <View class={`absolute inset-0 ${packClass("Hit", "vignette")}`} />}
      </For>
      <DamageDirection hits={props.hits ?? []} />
    </View>
  );
}

export const fixtures: Record<string, Partial<HitFeedbackProps>> = {
  default: { flash: "", critical: false, hits: [{ id: "h1", angle: 0.9, strength: 0.9, kind: "normal" }] },
  critical: { flash: "", critical: true, hits: [{ id: "h1", angle: -2.4, strength: 1, kind: "critical" }] },
};
