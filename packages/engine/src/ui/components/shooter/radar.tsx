import { For, Image, Line, Show, Text, View, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./shooter.en.ogt";

export interface RadarBlip {
  id: string;
  /** ally, foe or objective (each its own shape and colour). */
  kind?: string;
  /** On the map, 0 to 1 across. */
  x: number;
  /** On the map, 0 to 1 down. */
  y: number;
  /** Names it for a reader. */
  label?: string;
}

export interface ShooterRadarProps {
  /** The map texture (og.load.texture); 0 draws the dark well alone. */
  map?: Texture | 0;
  /** The map point under the player, 0 to 1 across. */
  centerX?: number;
  /** The map point under the player, 0 to 1 down. */
  centerY?: number;
  /** How much of the map the radar shows across, 0 to 1 (0.4). */
  zoom?: number;
  /** The player's heading in degrees clockwise from north: north stays up and the player's arrow turns. */
  heading?: number;
  /** What the radar shows. A blip off the radar's edge is clipped. Read only. */
  blips?: ReadonlyArray<RadarBlip>;
}

/**
 * The radar at the screen's top corner: the map in a dark round well with
 * a lit ring, a sweep and range rings over it, the squad and the enemies
 * the game reveals as blips, the objective as a gold diamond, the player's
 * arrow at the centre turned to the heading, and N on the ring's top. Read
 * only. Approximations: the sweep's radial mask has no TSX surface, so the
 * sweep fills the whole well; the glass's hairline range rings are dropped
 * (its vignette and top sheen stay); the objective's diamond clip is a
 * rotate-45 square; the player's arrow is stroked, its fill having no TSX
 * surface.
 */
const SIDE = 10.971; // rem: the radar's side, size-5xl * 0.8.

export function ShooterRadar(props: ShooterRadarProps) {
  const cx = () => props.centerX ?? 0.5;
  const cy = () => props.centerY ?? 0.5;
  const zoom = () => ((props.zoom ?? 0.4) > 0 ? props.zoom ?? 0.4 : 1);
  const mapLeft = () => `calc(${SIDE / 2}rem - ${cx() * (SIDE / zoom())}rem)`;
  const mapTop = () => `calc(${SIDE / 2}rem - ${cy() * (SIDE / zoom())}rem)`;
  return (
    <View
      class="relative size-[10.971rem] compact:size-[5.714rem] max-w-full text-fg"
      role="image"
      label={t("shooter.radar.name")}
    >
      <View class="absolute inset-0 overflow-hidden rounded-full bg-tile shadow-[0px_0px_0px_0.143rem_#12151ce6,0px_0px_0px_0.214rem_rgb(255_255_255/0.35),0px_0.714rem_2rem_rgb(0_0_0/0.6)]">
        <Show when={(props.map ?? 0) > 0}>
          <Image
            class="absolute"
            src={props.map ?? 0}
            style={{ width: `${SIDE / zoom()}rem`, height: `${SIDE / zoom()}rem`, left: mapLeft(), top: mapTop() }}
          />
        </Show>
        <View class="absolute inset-0 bg-[conic-gradient(from_-32deg,rgb(255_255_255/0.28),rgb(255_255_255/0.04)_64deg,transparent_64deg)]" />
        <View class="absolute left-1/2 top-1/2 size-[45%] -translate-x-1/2 -translate-y-1/2 rounded-full border border-white/14" />
        <View class="absolute left-1/2 top-1/2 size-[92%] -translate-x-1/2 -translate-y-1/2 rounded-full border border-white/14" />
        <For each={props.blips ?? []} key={(b) => b.id}>
          {(b) => (
            <View
              class={`absolute size-[0.643rem] compact:size-[0.5rem] -translate-x-1/2 -translate-y-1/2 data-ally:rounded-full data-ally:bg-[#4fd8ff] data-ally:shadow-[0px_0px_0px_0.143rem_#1b1f29cc] data-foe:rotate-45 data-foe:bg-danger data-foe:shadow-[0px_0px_0.571rem_#ff4d5ee6] data-objective:size-[0.929rem] data-objective:rotate-45 data-objective:bg-gold`}
              label={b().label}
              style={{
                left: `calc(${SIDE / 2}rem + ${(b().x - cx()) * (SIDE / zoom())}rem)`,
                top: `calc(${SIDE / 2}rem + ${(b().y - cy()) * (SIDE / zoom())}rem)`,
              }}
              data-ally={b().kind == "ally"}
              data-foe={b().kind == "foe"}
              data-objective={b().kind == "objective"}
            />
          )}
        </For>
        <View
          class="absolute"
          style={{
            width: "1rem",
            height: "1rem",
            left: "calc(50% - 0.5rem)",
            top: "calc(50% - 0.5rem)",
            transform: `rotate(${props.heading ?? 0}deg)`,
          }}
        >
          <Line points={[8, 0, 15, 16, 8, 12, 1, 16]} closed class="stroke-white stroke-[1.5]" />
        </View>
        <View class="absolute inset-0 rounded-full bg-[radial-gradient(circle,transparent_50%,rgb(0_0_0/0.55)_72%)]" />
        <View class="absolute left-0 top-0 right-0 h-[30%] bg-linear-to-b from-[rgb(255_255_255/0.16)] to-transparent" />
      </View>
      <Text
        class={`absolute top-0 left-1/2 -translate-x-1/2 py-0 px-1 text-[0.857rem] font-bold whitespace-nowrap text-fg nine-chip-cut bg-tile ${packClass("Chip", "frame")}`}
      >
        {t("shooter.radar.north")}
      </Text>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterRadarProps>> = {
  default: {"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.4,"heading":-20,"blips":[{"id":"a1","kind":"ally","x":0.436,"y":0.54,"label":"Moraine"},{"id":"a2","kind":"ally","x":0.532,"y":0.58,"label":"Quill"},{"id":"f1","kind":"foe","x":0.548,"y":0.412,"label":"Enemy"},{"id":"f2","kind":"foe","x":0.596,"y":0.444,"label":"Enemy"},{"id":"o1","kind":"objective","x":0.476,"y":0.388,"label":"Relay tower"}]},
  empty: {"map":0,"centerX":0.5,"centerY":0.5,"zoom":0.4,"heading":0,"blips":[]},
  edge: {"map":0,"centerX":0.05,"centerY":0.95,"zoom":0.8,"heading":135,"blips":[{"id":"far","kind":"foe","x":0.9,"y":0.1,"label":"Enemy"},{"id":"near","kind":"objective","x":0.1,"y":0.9,"label":"Drop"}]},
};
