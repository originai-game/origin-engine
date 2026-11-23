import { og, type ParticleEmitter } from "@origingame/engine";
import { For, Icon, Particles, Show, Text, View, useWorld } from "@origingame/engine/ui";
import charge from "./fx/charge.ogt";
import chargeGlow from "./fx/charge-glow.ogt";
import coinGlints from "./fx/coin-glints.ogt";
import coins from "./fx/coins.ogt";
import flash from "./fx/flash.ogt";
import glints from "./fx/glints.ogt";
import lightColumn from "./fx/light-column.ogt";
import motes from "./fx/motes.ogt";
import rays from "./fx/rays.ogt";
import rise from "./fx/rise.ogt";
import shards from "./fx/shards.ogt";
import shockRing from "./fx/shock-ring.ogt";
import twinkle from "./fx/twinkle.ogt";
import wash from "./fx/wash.ogt";

export interface FlourishProps {
  /** Which one: reward (stars, rays and a ring of light thrown from it), levelUp (a column of light, a screen wash, stars rising and a ring), purchase (a fountain of spinning coins, the target popping), reveal (a charge drawn in, then a rarity's burst: richer from common to legendary) or highlight (an aura in the pack's primary colour pulsing, a sheen sweeping it and stars twinkling, while it shows). */
  kind: "reward" | "levelUp" | "purchase" | "reveal" | "highlight";
  /** Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once. highlight needs none. */
  shots?: ReadonlyArray<{ id: string }>;
  /** reveal: the rarity whose colour and tier it takes (common, uncommon, rare, epic, legendary, mythic); empty is legendary. */
  rarity?: string;
  /** What it decorates (an icon, a slot, a price, a button); the effect is drawn over and around it. */
  children?: any;
}

/** A built-in emitter in the mount's world; 0, which draws nothing, when the bundle does not hold it. */
function emitter(path: string): ParticleEmitter | 0 {
  const loaded = og.load.emitter(useWorld(), path);
  return loaded.ok ? loaded.value : 0;
}

/** The rarities by tier: a reveal throws rays from rare, a second set of rays and more glints from epic, and from legendary a third set, more shards, a column of light and a screen wash. */
const RARITIES = ["common", "uncommon", "rare", "epic", "legendary", "mythic"];

/** The pack event each kind sounds as it plays. */
const SOUND: Record<string, "reward" | "levelup" | "purchase"> = { reward: "reward", levelUp: "levelup", purchase: "purchase", reveal: "reward" };

/** reveal: seconds of charge before the burst (the target's `animate-charge` gathers over the same 40% of its 0.6 s). */
const BURST = 0.24;

/**
 * A flourish around what it decorates, the engine's UI effect presets: a
 * reward's flash, rays, ring and stars; a level-up's column of light,
 * screen wash and rising stars; a purchase's fountain of spinning coins
 * catching the light while the price pops; a rarity reveal that draws a
 * charge in (glints gathering, a glow swelling, the target growing), then
 * bursts (a flash, rays, a ring and shards, the target popping) and
 * settles in lingering motes, each tier richer (rays from rare, double
 * rays from epic, triple rays, a column and a screen wash from
 * legendary); the beams draw behind the target, the rest over it; or a highlighted
 * button's aura in the pack's primary colour, sweeping sheen and twinkling
 * stars. Each play runs once per shot with its pack sound (highlight runs
 * while it shows, silent). The particles are the engine's emitters
 * (./fx/*.ogt) with the shared textures (`@styles/shared/fx/`), drawn by
 * `Particles`, one node per ink, so a pack's colour roles colour them;
 * reduced motion draws none.
 */
export function Flourish(props: FlourishProps) {
  const fx = {
    charge: emitter(charge), chargeGlow: emitter(chargeGlow), coinGlints: emitter(coinGlints), coins: emitter(coins),
    flash: emitter(flash), glints: emitter(glints), column: emitter(lightColumn), motes: emitter(motes), rays: emitter(rays),
    rise: emitter(rise), shards: emitter(shards), ring: emitter(shockRing), twinkle: emitter(twinkle), wash: emitter(wash),
  };
  const kind = () => props.kind ?? "reward";
  const rarity = () => (RARITIES.includes(props.rarity ?? "") ? props.rarity! : "legendary");
  const tier = () => RARITIES.indexOf(rarity());
  const ink = () => `text-${rarity()}`;
  const shots = () => (kind() == "highlight" ? [] : (props.shots ?? []));
  /** The target's motion per play: a reveal's charge and pop, a purchase's pop; it replays with each new shot. */
  const pose = () => (kind() == "reveal" ? "animate-charge" : kind() == "purchase" ? "animate-beat" : "");
  const latest = () => {
    const all = shots();
    return pose() && all.length ? [all[all.length - 1]!] : [];
  };
  return (
    <View class="items-center">
      <View class="relative items-center justify-center">
        <Show when={kind() == "highlight"}>
          <View class="absolute inset-0.5 rounded-panel bg-art-primary opacity-40 shadow-glow-lg shadow-art-primary animate-pulse" />
        </Show>
        <For each={shots()} key={(s) => s.id}>
          {() => (
            <View class="absolute inset-0 items-center justify-center">
              <Show when={kind() == "reward"}>
                <Particles class="absolute size-2 text-gold" emitter={fx.rays} seed={2} />
              </Show>
              <Show when={kind() == "levelUp"}>
                <Particles class="absolute size-2 text-gold" emitter={fx.wash} seed={1} />
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.flash} seed={2} />
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.ring} seed={5} />
                <Particles class="absolute size-2 text-gold" emitter={fx.column} seed={3} />
                <Particles class="absolute size-2 text-gold" emitter={fx.rays} seed={4} />
              </Show>
              <Show when={kind() == "reveal"}>
                <Show when={tier() >= 4}>
                  <Particles class={`absolute size-2 ${ink()}`} emitter={fx.wash} seed={4} delay={BURST} />
                  <Particles class={`absolute size-2 ${ink()}`} emitter={fx.column} seed={5} delay={BURST} />
                  <Particles class={`absolute size-2 ${ink()}`} emitter={fx.rays} seed={15} delay={BURST} />
                </Show>
                <Show when={tier() >= 3}>
                  <Particles class={`absolute size-2 ${ink()}`} emitter={fx.rays} seed={6} delay={BURST} />
                </Show>
                <Show when={tier() >= 2}>
                  <Particles class={`absolute size-2 ${ink()}`} emitter={fx.rays} seed={8} delay={BURST} />
                </Show>
              </Show>
            </View>
          )}
        </For>
        <Show when={latest().length == 0}>{props.children}</Show>
        <For each={latest()} key={(s) => s.id}>
          {() => <View class={pose()}>{props.children}</View>}
        </For>
        <Show when={kind() == "highlight"}>
          <View class="absolute inset-x-0.5 top-0.5 bottom-1.5 rounded-panel overflow-hidden">
            <View class="absolute left-0 inset-y-0 w-1/3 bg-linear-to-r from-transparent via-sheen to-transparent animate-shimmer" />
          </View>
          <Particles class="absolute -inset-1 text-fg" emitter={fx.twinkle} seed={1} />
        </Show>
        <For each={shots()} key={(s) => s.id}>
          {() => (
            <View sound={SOUND[kind()]} class="absolute inset-0 items-center justify-center">
              <Show when={kind() == "reward"}>
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.flash} seed={1} />
                <Particles class="absolute size-2 text-gold" emitter={fx.ring} seed={3} />
                <Particles class="absolute size-2 text-gold" emitter={fx.glints} seed={4} />
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.glints} seed={5} />
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.motes} seed={6} delay={0.3} />
              </Show>
              <Show when={kind() == "levelUp"}>
                <Particles class="absolute size-2 text-gold" emitter={fx.glints} seed={6} />
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.glints} seed={7} />
                <View class="absolute -bottom-4 -inset-x-24 h-2">
                  <Particles class="absolute inset-0 text-gold-hi" emitter={fx.rise} seed={8} />
                  <Particles class="absolute inset-0 text-gold" emitter={fx.rise} seed={9} />
                </View>
              </Show>
              <Show when={kind() == "purchase"}>
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.flash} seed={1} />
                <Particles class="absolute size-2 text-gold" emitter={fx.ring} seed={2} />
                <Particles class="absolute size-2 text-gold" emitter={fx.coins} seed={3} />
                <Particles class="absolute size-2 text-gold" emitter={fx.coins} seed={4} />
                <Particles class="absolute size-2 text-fg" emitter={fx.coinGlints} seed={5} />
                <Particles class="absolute size-2 text-gold-hi" emitter={fx.glints} seed={6} />
              </Show>
              <Show when={kind() == "reveal"}>
                <Particles class={`absolute size-2 ${ink()}`} emitter={fx.chargeGlow} seed={1} />
                <Particles class={`absolute size-2 ${ink()}`} emitter={fx.charge} seed={2} />
                <Particles class="absolute size-2 text-fg" emitter={fx.charge} seed={3} />
                <Particles class={`absolute size-2 ${ink()}`} emitter={fx.flash} seed={9} delay={BURST} />
                <Particles class={`absolute size-2 ${ink()}`} emitter={fx.ring} seed={10} delay={BURST} />
                <Particles class={`absolute size-2 ${ink()}`} emitter={fx.shards} seed={11} delay={BURST} />
                <Particles class="absolute size-2 text-fg" emitter={fx.glints} seed={12} delay={BURST} />
                <Show when={tier() >= 3}>
                  <Particles class={`absolute size-2 ${ink()}`} emitter={fx.glints} seed={14} delay={BURST} />
                </Show>
                <Show when={tier() >= 4}>
                  <Particles class={`absolute size-2 ${ink()}`} emitter={fx.shards} seed={7} delay={BURST} />
                </Show>
                <Particles class={`absolute size-2 ${ink()}`} emitter={fx.motes} seed={13} delay={BURST + 0.3} />
              </Show>
            </View>
          )}
        </For>
      </View>
    </View>
  );
}

const ICON = (name: string) => () => (
  <View class="size-24 items-center justify-center nine-slot"><Icon class="size-16" name={name} /></View>
);

export const fixtures: Record<string, Partial<FlourishProps>> = {
  reward: { kind: "reward", shots: [{ id: "1" }], children: ICON("chest") },
  levelUp: { kind: "levelUp", shots: [{ id: "1" }], children: () => <View class="px-6 py-2 rounded-panel bg-bg nine-plate"><Text class="text-display-md font-bold text-gold">LV 12</Text></View> },
  purchase: { kind: "purchase", shots: [{ id: "1" }], children: () => <View class="flex-row items-center gap-1 pl-2 pr-4 h-10 nine-plate"><Icon class="size-7" name="coin" /><Text class="text-lg font-bold text-art-fg">12,480</Text></View> },
  reveal: { kind: "reveal", rarity: "epic", shots: [{ id: "1" }], children: ICON("gem") },
  revealLegendary: { kind: "reveal", rarity: "legendary", shots: [{ id: "1" }], children: ICON("crown") },
  highlight: { kind: "highlight", children: () => <View class="h-14 w-56 items-center justify-center pb-1 nine-button-primary"><Text class="text-lg font-bold text-art-primary-fg">Claim</Text></View> },
};
