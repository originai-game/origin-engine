import { og, type ParticleEmitter } from "@origingame/engine";
import { For, Particles, Show, Text, View, t, useWorld } from "@origingame/engine/ui";
import "./celebration.en.ogt";
import confetti from "./fx/confetti.ogt";
import embers from "./fx/embers.ogt";
import firework from "./fx/firework.ogt";
import sparks from "./fx/sparks.ogt";

export interface CelebrationProps {
  /** Which one: confetti (falling over its box from the top edge), burst (a ring and sparks thrown from its centre), levelUp (a ring of light behind the title and the new level, embers rising under them) or fireworks (shells bursting across its top). */
  kind: string;
  /** Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once. */
  shots?: ReadonlyArray<{ id: string }>;
  /** levelUp: the big word; empty is Level up. */
  title?: string;
  /** levelUp: the new level (12); empty is none. */
  value?: string;
}

/** A built-in emitter in the mount's world; 0, which draws nothing, when the bundle does not hold it. */
function emitter(path: string): ParticleEmitter | 0 {
  const loaded = og.load.emitter(useWorld(), path);
  return loaded.ok ? loaded.value : 0;
}

/** Where fireworks' five shells burst (down from the top edge), in which ink, and how long after the play, in seconds. */
const SHELLS = [
  { drop: "mt-12", ink: "text-gold", delay: 0 },
  { drop: "mt-6", ink: "text-hud-accent-ink", delay: 0.8 },
  { drop: "mt-24", ink: "text-danger", delay: 0.2 },
  { drop: "mt-8", ink: "text-success", delay: 0.4 },
  { drop: "mt-18", ink: "text-gold", delay: 0.6 },
];

/**
 * A reward over the HUD: confetti thrown from the top edge, a burst's ring
 * and sparks, a level-up's ring of light with its word in gold, its new
 * level and embers rising under it, or fireworks' five shells. Each play
 * runs once per shot, with the pack's reward sound. The particles are the
 * engine's emitters drawn by `Particles` (./fx/*.ogt), one node per ink,
 * so a pack's roles colour them; reduced motion draws none. The
 * level-up's rays wait for pack art.
 */
export function Celebration(props: CelebrationProps) {
  const kind = () => props.kind ?? "";
  const fx = { confetti: emitter(confetti), sparks: emitter(sparks), embers: emitter(embers), firework: emitter(firework) };
  return (
    <View class="absolute inset-0 text-fg text-shadow-2xs">
      <For each={props.shots ?? []} key={(s) => s.id}>
        {() => (
          <View sound={kind() == "levelUp" ? "levelup" : "reward"} class="absolute inset-0 flex-col items-center justify-center">
            <Show when={kind() == "confetti"}>
              <View class="absolute top-0 inset-x-0 h-1">
                <Particles class="absolute inset-0 text-hud-accent-ink" emitter={fx.confetti} seed={1} />
                <Particles class="absolute inset-0 text-gold" emitter={fx.confetti} seed={2} />
                <Particles class="absolute inset-0 text-success" emitter={fx.confetti} seed={3} />
                <Particles class="absolute inset-0 text-info" emitter={fx.confetti} seed={4} />
              </View>
            </Show>
            <Show when={kind() == "levelUp"}>
              <View class="relative w-105 max-w-full min-h-72 flex-col items-center justify-center">
                <View class="absolute size-66 rounded-full border-4 border-gold shadow-glow-lg shadow-gold inset-shadow-glow inset-shadow-gold bg-glass animate-stamp compact:size-23.75" />
                <Text class="relative max-w-full mb-3 text-title font-bold tracking-label uppercase text-center text-gold animate-stamp">
                  {props.title || t("component.celebration.level-up")}
                </Text>
                <Show when={props.value}>
                  <Text class="relative text-8xl font-bold tabular-nums leading-flat whitespace-nowrap text-fg shadow-none animate-pop compact:text-display-md">{props.value}</Text>
                </Show>
                <View class="absolute bottom-8 w-60 max-w-full h-2">
                  <Particles class="absolute inset-0 text-gold" emitter={fx.embers} seed={1} />
                  <Particles class="absolute inset-0 text-warn" emitter={fx.embers} seed={2} />
                  <Particles class="absolute inset-0 text-danger" emitter={fx.embers} seed={3} />
                </View>
              </View>
            </Show>
            <Show when={kind() == "burst"}>
              <View class="size-16 rounded-full border-4 border-rim-strong shadow-glow-sm shadow-knob animate-ping" />
              <View class="absolute size-2">
                <Particles class="absolute inset-0 text-hud-accent-ink" emitter={fx.sparks} seed={1} />
                <Particles class="absolute inset-0 text-gold" emitter={fx.sparks} seed={2} />
                <Particles class="absolute inset-0 text-fg" emitter={fx.sparks} seed={3} />
              </View>
            </Show>
            <Show when={kind() == "fireworks"}>
              <View class="absolute top-0 inset-x-0 h-60 flex-row justify-between py-0 px-12">
                <For each={SHELLS} key={(s) => s.drop}>
                  {(s, i) => <Particles class={`size-2 ${s().ink} ${s().drop}`} emitter={fx.firework} seed={i() + 1} delay={s().delay} />}
                </For>
              </View>
            </Show>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<CelebrationProps>> = {
  default: {"kind":"levelUp","shots":[{"id":"1"}],"title":"","value":"12"},
  confetti: {"kind":"confetti","shots":[{"id":"1"}]},
  burst: {"kind":"burst","shots":[{"id":"1"}]},
  fireworks: {"kind":"fireworks","shots":[{"id":"1"}]},
};
